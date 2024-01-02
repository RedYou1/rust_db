extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(Binary)]
pub fn binary_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    let struct_name = &ast.ident;

    let data_struct = match ast.data {
        syn::Data::Struct(ref data_struct) => data_struct,
        _ => panic!("Binary derive only supports structs."),
    };

    let mut field_declarations = Vec::new();
    let mut from_bin_assignments = Vec::new();
    let mut into_bin_statements = Vec::new();
    let mut bin_size_statements = Vec::new();
    let mut delete_statements = Vec::new();

    for field in data_struct.fields.iter() {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        field_declarations.push(quote::quote! {
            #field_name: #field_name,
        });

        from_bin_assignments.push(quote::quote! {
            let #field_name = <#field_type>::from_bin(&data[offset..], path)?;
            offset += <#field_type>::bin_size();
        });

        into_bin_statements.push(quote::quote! {
            bin_data.extend_from_slice(&self.#field_name.into_bin(path)?);
        });

        bin_size_statements.push(quote::quote! {
            size += <#field_type>::bin_size();
        });

        delete_statements.push(quote::quote! {
            self.#field_name.delete(path)?;
        });
    }

    quote::quote! {
        impl Binary for #struct_name {
            fn from_bin(data: &[u8], path: &str) -> std::io::Result<Self> {
                let mut offset = 0;
                #(#from_bin_assignments)*
                Ok(#struct_name {
                    #(#field_declarations)*
                })
            }

            fn into_bin(&self, path: &str) -> std::io::Result<Vec<u8>> {
                let mut bin_data = Vec::new();
                #(#into_bin_statements)*
                Ok(bin_data)
            }

            fn bin_size() -> usize {
                let mut size = 0;
                #(#bin_size_statements)*
                size
            }

            fn delete(&self, path: &str) -> std::io::Result<()>{
                #(#delete_statements)*
                Ok(())
            }
        }
    }
    .into()
}

#[proc_macro_derive(TableRow, attributes(PrimaryKey))]
pub fn table_row_macro(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let fields = match ast.data {
        Data::Struct(ref data) => &data.fields,
        _ => panic!("TableRow derive only supports structs."),
    };

    let field = fields
        .iter()
        .find(|field| {
            field
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident("PrimaryKey"))
        })
        .expect("struct must contains the attribute PrimaryKey");

    let primary_field_name = field.ident.as_ref().expect("Field must have an identifier");
    let primary_field_type = &field.ty;

    let struct_name = &ast.ident;

    let mut field_declarations = Vec::new();
    let mut from_bin_assignments = Vec::new();
    let mut into_bin_statements = Vec::new();
    let mut bin_size_statements = Vec::new();
    let mut delete_statements = Vec::new();

    for field in fields.iter() {
        let field_name = field.ident.as_ref().expect("Field must have an identifier");
        if primary_field_name.eq(field_name) {
            continue;
        }
        let field_type = &field.ty;

        field_declarations.push(quote::quote! {
            #field_name: #field_name,
        });

        from_bin_assignments.push(quote::quote! {
            let #field_name = <#field_type>::from_row_bin(&data[offset..], &#primary_field_name, path)?;
            offset += <#field_type>::row_bin_size(PhantomData::<#primary_field_type>::default());
        });

        into_bin_statements.push(quote::quote! {
            bin_data.extend_from_slice(&self.#field_name.into_row_bin(&self.#primary_field_name, path)?);
        });

        bin_size_statements.push(quote::quote! {
            size += <#field_type>::row_bin_size(PhantomData::<#primary_field_type>::default());
        });

        delete_statements.push(quote::quote! {
            self.#field_name.row_delete(&self.#primary_field_name, path)?;
        });
    }

    quote::quote! {
        impl Binary for #struct_name {
            fn from_bin(data: &[u8], path: &str) -> std::io::Result<Self> {
                let #primary_field_name = <#primary_field_type>::from_bin(&data, path)?;
                let mut offset = <#primary_field_type>::bin_size();
                #(#from_bin_assignments)*
                Ok(#struct_name {
                    #primary_field_name: #primary_field_name,
                    #(#field_declarations)*
                })
            }

            fn into_bin(&self, path: &str) -> std::io::Result<Vec<u8>> {
                let mut bin_data = self.#primary_field_name.into_bin(path)?;
                #(#into_bin_statements)*
                Ok(bin_data)
            }

            fn bin_size() -> usize {
                let mut size = <#primary_field_type>::bin_size();
                #(#bin_size_statements)*
                size
            }

            fn delete(&self, path: &str) -> std::io::Result<()>{
                self.#primary_field_name.delete(path)?;
                #(#delete_statements)*
                Ok(())
            }
        }

        impl TableRow<#primary_field_type> for #struct_name {
            fn id(&self) -> &#primary_field_type {
                &self.#primary_field_name
            }
        }
    }
    .into()
}
