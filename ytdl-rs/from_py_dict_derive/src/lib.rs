extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::Data::Struct;
use syn::Fields::Named;
use syn::Type::Path;

fn impl_from_py_dict_derive(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let gen: TokenStream;
    match &ast.data {
        Struct(data) => {
            match &data.fields {
                Named(fields_named) => {
                    let names: Vec<syn::Ident> = fields_named.named.clone().into_iter().map(|field| field.ident.clone().unwrap()).collect();
                    let types: Vec<syn::Ident> = fields_named.named.clone().into_iter().flat_map(|field| {
                        match &field.ty {
                            Path(type_path) => {
                                vec![type_path.path.segments[0].ident.clone()]
                            },
                            _ => vec![]
                        }
                    }).collect();

                    gen = quote! {
                        impl FromPyDict for #name {
                            fn from_py_dict(dict: &pyo3::types::PyDict) -> pyo3::PyResult<Self> {
                                #(
                                let #names: #types = dict.get_item(stringify!(#names)).unwrap().extract::<#types>()?;
                                )*

                                Ok(Self {
                                    #(
                                    #names,
                                    )*
                                })
                            }

                            fn from_py_dict_list(list: &pyo3::types::PyList) -> pyo3::PyResult<Vec<Self>> {
                                let mut out: Vec<Self> = Vec::new();
                                for item in list {
                                    let dict_result = item.downcast::<pyo3::types::PyDict>();
                                    out.push(Self::from_py_dict(dict_result.unwrap())?);
                                }

                                Ok(out)
                            }
                        }
                    }.into();

                },
                _ => {
                    println!("Wrong input data");
                    gen = quote! {}.into();
                }
            }
        },
        _ => {
            println!("Wrong input data");
            gen = quote! {}.into();
        }
    }
    gen.into()
}

#[proc_macro_derive(FromPyDict)]
pub fn from_py_dict_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_from_py_dict_derive(&ast)
}