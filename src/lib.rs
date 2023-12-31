extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, Data, DeriveInput};

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
        let field_ident = match &field.ident {
            Some(field_ident) => field_ident,
            _ => continue,
        };

        let field_type = &field.ty;

        field_declarations.push(quote::quote! {
            #field_ident: #field_ident,
        });

        from_bin_assignments.push(quote::quote! {
            let #field_ident = <#field_type>::from_bin(&data[offset..], path)?;
            offset += <#field_type>::bin_size();
        });

        into_bin_statements.push(quote::quote! {
            bin_data.extend_from_slice(&self.#field_ident.into_bin(path)?);
        });

        bin_size_statements.push(quote::quote! {
            size += <#field_type>::bin_size();
        });

        delete_statements.push(quote::quote! {
            self.#field_ident.delete(path)?;
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

fn has_primary_key(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("PrimaryKey"))
}

#[proc_macro_derive(IsRow, attributes(PrimaryKey))]
pub fn is_row_macro(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;

    let fields = match ast.data {
        Data::Struct(ref data) => &data.fields,
        _ => panic!("IsRow derive only supports structs."),
    };

    let field = fields
        .iter()
        .find(|field| has_primary_key(&field.attrs))
        .expect("struct must contains the attribute PrimaryKey");

    let field_name = field.ident.as_ref().expect("Field must have an identifier");
    let field_type = &field.ty;

    quote! {
        impl IsRow<#field_type> for #name {
            fn id(&self) -> &#field_type {
                &self.#field_name
            }
        }
    }
    .into()
}
