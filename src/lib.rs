extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro_derive(Binary)]
pub fn binary_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_binary_for_struct(&ast)
}

fn impl_binary_for_struct(ast: &syn::DeriveInput) -> TokenStream {
    let struct_name = &ast.ident;

    if let syn::Data::Struct(ref data_struct) = ast.data {
        let mut field_declarations = Vec::new();
        let mut from_bin_assignments = Vec::new();
        let mut into_bin_statements = Vec::new();
        let mut bin_size_statements = Vec::new();
        let mut delete_statements = Vec::new();

        for field in data_struct.fields.iter() {
            if let Some(field_ident) = &field.ident {
                let field_type = &field.ty;

                field_declarations.push(quote::quote! {
                    #field_ident: #field_ident,
                });

                from_bin_assignments.push(quote::quote! {
                    let #field_ident = <#field_type>::from_bin(&data[_offset..], path);
                    _offset += <#field_type>::bin_size();
                });

                into_bin_statements.push(quote::quote! {
                    bin_data.extend_from_slice(&self.#field_ident.into_bin(path));
                });

                bin_size_statements.push(quote::quote! {
                    size += <#field_type>::bin_size();
                });

                delete_statements.push(quote::quote! {
                    self.#field_ident.delete(path);
                });
            }
        }

        let gen = quote::quote! {
            impl Binary for #struct_name {
                fn from_bin(data: &[u8], path: &str) -> Self {
                    let mut _offset = 0;
                    #(#from_bin_assignments)*
                    #struct_name {
                        #(#field_declarations)*
                    }
                }

                fn into_bin(&self, path: &str) -> Vec<u8> {
                    let mut bin_data = Vec::new();
                    #(#into_bin_statements)*
                    bin_data
                }

                fn bin_size() -> usize {
                    let mut size = 0;
                    #(#bin_size_statements)*
                    size
                }

                fn delete(&self, path: &str) {
                    #(#delete_statements)*
                }
            }
        };

        return gen.into();
    }

    panic!("Binary derive only supports structs.")
}
