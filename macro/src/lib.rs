extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{Data, DeriveInput, parse_macro_input};

/// # Panics
/// Will panic if cant parse the input
#[proc_macro_derive(Binary)]
pub fn binary_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).expect("failed to parse the input");
    let struct_name = &ast.ident;

    let syn::Data::Struct(ref data_struct) = ast.data else {
        panic!("Binary derive only supports structs.")
    };
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let mut field_declarations = Vec::new();
    let mut from_bin_assignments = Vec::new();
    let mut as_bin_statements = Vec::new();
    let mut bin_size_statements = Vec::new();
    let mut delete_statements = Vec::new();

    for field in &data_struct.fields {
        let field_name = field.ident.as_ref().expect("Field must have an identifier");
        let field_type = &field.ty;

        field_declarations.push(quote! {
            #field_name: #field_name,
        });

        from_bin_assignments.push(quote! {
            let #field_name = <#field_type>::from_bin(&_data[offset..], _path)?;
            offset += <#field_type>::bin_size();
        });

        as_bin_statements.push(quote! {
            bin_data.extend_from_slice(&self.#field_name.as_bin(_path)?);
        });

        bin_size_statements.push(quote! {
            size += <#field_type>::bin_size();
        });

        delete_statements.push(quote! {
            self.#field_name.delete(_path)?;
        });
    }

    quote! {
        impl #impl_generics Binary for #struct_name #ty_generics #where_clause {
            fn from_bin(_data: &[u8], _path: &BDPath) -> std::io::Result<Self> {
                let mut offset = 0;
                #(#from_bin_assignments)*
                Ok(#struct_name {
                    #(#field_declarations)*
                })
            }

            fn as_bin(&mut self, _path: &BDPath) -> std::io::Result<Vec<u8>> {
                let mut bin_data = Vec::new();
                #(#as_bin_statements)*
                Ok(bin_data)
            }

            fn bin_size() -> usize {
                let mut size = 0;
                #(#bin_size_statements)*
                size
            }

            fn delete(&self, _path: &BDPath) -> std::io::Result<()>{
                #(#delete_statements)*
                Ok(())
            }
        }
    }
    .into()
}

/// # Panics
/// Will panic if cant parse the input
#[expect(clippy::too_many_lines)]
#[proc_macro_derive(Table, attributes(PrimaryKey, Index, Unique))]
pub fn table_row_macro(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let fields = match ast.data {
        Data::Struct(ref data) => &data.fields,
        _ => panic!("Table derive only supports structs."),
    };

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let primary_field = fields
        .iter()
        .find(|field| {
            field
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident("PrimaryKey"))
        })
        .expect("struct must contains the attribute PrimaryKey");

    let primary_field_name = primary_field
        .ident
        .as_ref()
        .expect("Field must have an identifier");
    let primary_field_type = &primary_field.ty;

    let struct_name = &ast.ident;

    let mut field_declarations = Vec::new();
    let mut from_bin_assignments = Vec::new();
    let mut as_bin_statements = Vec::new();
    let mut bin_size_statements = Vec::new();
    let mut delete_statements = Vec::new();
    let mut get_indexes_statements = Vec::new();
    let mut get_indexes_functions_signature = Vec::new();
    let mut get_indexes_functions = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().expect("Field must have an identifier");
        if primary_field_name.eq(field_name) {
            continue;
        }
        let field_type = &field.ty;

        field_declarations.push(quote! {
            #field_name: #field_name,
        });

        from_bin_assignments.push(quote! {
            let #field_name = <#field_type>::from_bin(&_data[offset..], _path)?;
            offset += <#field_type>::bin_size();
        });

        as_bin_statements.push(quote! {
            bin_data.extend_from_slice(&self.#field_name.as_bin(_path)?);
        });

        bin_size_statements.push(quote! {
            size += <#field_type>::bin_size();
        });

        delete_statements.push(quote! {
            self.#field_name.delete(_path)?;
        });

        let index = field.attrs.iter().any(|attr| attr.path().is_ident("Index"));
        let unique = field
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("Unique"));

        if index || unique {
            let in_name = format!("{field_name}");
            get_indexes_statements.push(quote! {
                Box::new(IndexFile::new(BDPath::new_index(path.clone(), #in_name.to_owned()), Box::new(|row: &#struct_name| &row.#field_name), #unique)?),
            });

            let fn_name = Ident::new(format!("get_by_{field_name}").as_str(), Span::call_site());

            let i = get_indexes_functions.len();

            if unique {
                get_indexes_functions_signature.push(quote! {
                    fn #fn_name(&self, col: &#field_type) -> TableGet<Option<#struct_name>>;
                });
                get_indexes_functions.push(quote! {
                    fn #fn_name(&self, col: &#field_type) -> TableGet<Option<#struct_name>> {
                        match &match unsafe{self.get_index_file::<#field_type>(#i)}.indx(col) {
                            IndexGet::Found(_, index) => index,
                            IndexGet::NotFound(_) => return TableGet::NotFound,
                            IndexGet::InternalError(e) => return TableGet::InternalError(e),
                            IndexGet::Err(e) => return TableGet::Err(e),
                        }[..]
                        {
                            [] => TableGet::Found(None),
                            [data] => {
                                match self.get_by_index(data.index) {
                                    Ok(data) => TableGet::Found(Some(data)),
                                    Err(e) => TableGet::Err(e),
                                }
                            },
                            _ => TableGet::InternalError("multiple with the same id".to_owned()),
                        }
                    }
                });
            } else {
                get_indexes_functions_signature.push(quote! {
                    fn #fn_name(&self, col: &#field_type) -> TableGet<Vec<#struct_name>>;
                });
                get_indexes_functions.push(quote! {
                    fn #fn_name(&self, col: &#field_type) -> TableGet<Vec<#struct_name>> {
                        let index = match unsafe{self.get_index_file::<#field_type>(#i)}.indx(col) {
                            IndexGet::Found(_, index) => index,
                            IndexGet::NotFound(_) => return TableGet::NotFound,
                            IndexGet::InternalError(e) => return TableGet::InternalError(e),
                            IndexGet::Err(e) => return TableGet::Err(e),
                        };
                        match index
                            .into_iter()
                            .map(|index| self.get_by_index(index.index))
                            .collect::<io::Result<Vec<#struct_name>>>()
                        {
                            Ok(datas) => TableGet::Found(datas),
                            Err(e) => TableGet::Err(e),
                        }
                    }
                });
            }
        }
    }

    let trait_name = Ident::new(
        format!("{struct_name}TableFileGets").as_str(),
        Span::call_site(),
    );

    quote! {
        impl #impl_generics Binary for #struct_name #ty_generics #where_clause {
            fn from_bin(_data: &[u8], _path: &BDPath) -> std::io::Result<Self> {
                let #primary_field_name = <#primary_field_type>::from_bin(&_data, _path)?;
                let mut offset = <#primary_field_type>::bin_size();
                #(#from_bin_assignments)*
                Ok(#struct_name {
                    #primary_field_name: #primary_field_name,
                    #(#field_declarations)*
                })
            }

            fn as_bin(&mut self, _path: &BDPath) -> std::io::Result<Vec<u8>> {
                let mut bin_data = self.#primary_field_name.as_bin(_path)?;
                #(#as_bin_statements)*
                Ok(bin_data)
            }

            fn bin_size() -> usize {
                let mut size = <#primary_field_type>::bin_size();
                #(#bin_size_statements)*
                size
            }

            fn delete(&self, _path: &BDPath) -> std::io::Result<()>{
                self.#primary_field_name.delete(_path)?;
                #(#delete_statements)*
                Ok(())
            }
        }

        impl #impl_generics Table for #struct_name #ty_generics #where_clause {
            type ID = #primary_field_type;

            fn id(&self) -> &#primary_field_type {
                &self.#primary_field_name
            }

            fn get_indexes(path: String) -> io::Result<Vec<Box<dyn UnspecifiedIndex<#struct_name>>>>{
                Ok(vec![
                    #(#get_indexes_statements)*
                ])
            }
        }

        trait #trait_name {
            #(#get_indexes_functions_signature)*
        }
        impl #trait_name for TableFile<#struct_name> {
            #(#get_indexes_functions)*
        }
    }
    .into()
}
