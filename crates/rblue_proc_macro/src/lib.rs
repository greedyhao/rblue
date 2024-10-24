// src/lib.rs
extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Type};

#[proc_macro_derive(ToU8Array)]
pub fn derive_to_u8_array(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = match &input.data {
        Data::Struct(data_struct) => {
            let fields = match &data_struct.fields {
                Fields::Named(fields_named) => &fields_named.named,
                _ => unimplemented!(),
            };

            // let field_names = fields.iter().map(|f| &f.ident);
            // let field_types = fields.iter().map(|f| &f.ty);
            let conversions = fields.iter().map(|f| {
                let name = &f.ident;
                let ty = &f.ty;
                match ty {
                    Type::Path(type_path) => {
                        // eprintln!("Type: {:?}", type_path.path.segments.first().unwrap().ident);
                        if type_path.path.is_ident("BDAddr")
                            || type_path.path.is_ident("SupportedCommands")
                            || type_path.path.is_ident("LMPFeatures")
                            || type_path.path.is_ident("LEAdvPacket")
                        {
                            quote! {
                                array.extend(self.#name);
                            }
                        } else if type_path.path.is_ident("bool") {
                            quote! {
                                array.push(self.#name as u8);
                            }
                        } else {
                            quote! {
                                array.extend(&self.#name.to_le_bytes());
                            }
                        }
                    }
                    _ => {
                        quote! {
                            array.extend(&self.#name.to_le_bytes());
                        }
                    }
                }
            });

            quote! {
                impl RBlueToU8Array for #name {
                    fn to_u8_array(&self) -> Vec<u8> {
                        let mut array = Vec::new();
                        #(#conversions)*
                        array
                    }
                }
            }
        }
        Data::Enum(_) => {
            quote! {
                impl RBlueToU8Array for #name {
                    fn to_u8_array(&self) -> Vec<u8> {
                        Vec::from(self.to_le_bytes())
                    }
                }
            }
        }
        _ => unimplemented!(),
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(EnumU8ToLeBytes)]
pub fn derive_enum_u8_to_le_bytes(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Get the name of the enum
    let name = &input.ident;

    // Ensure the input is an enum
    let variants = if let Data::Enum(data_enum) = &input.data {
        &data_enum.variants
    } else {
        panic!("#[derive(ToLeBytes)] is only defined for enums");
    };

    // Generate match arms for each variant
    let match_arms = variants.iter().map(|variant| {
        let ident = &variant.ident;
        quote! {
            Self::#ident => [Self::#ident as u8],
        }
    });

    // Generate the implementation
    let expanded = quote! {
        impl #name {
            pub fn to_le_bytes(&self) -> [u8; 1] {
                match self {
                    #(#match_arms)*
                }
            }
        }
    };

    // Convert the expanded implementation back into a TokenStream and return it
    TokenStream::from(expanded)
}

#[proc_macro_derive(FromBytes)]
pub fn from_bytes_derive(input: TokenStream) -> TokenStream {
    // 解析输入的 Rust 代码为 AST
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = match input.data {
        Data::Struct(data) => {
            let field_parsers = match data.fields {
                Fields::Named(ref fields) => {
                    fields.named.iter().enumerate().map(|(i, f)| {
                        let name = &f.ident;
                        let ty = &f.ty;
                        if quote!(#ty).to_string() == "bool" {
                            quote! {
                                #name: bytes[#i] != 0
                            }
                        } else {
                            quote! {
                                #name: <#ty>::from_le_bytes(bytes[#i..#i+core::mem::size_of::<#ty>()].try_into().ok()?)
                            }
                        }
                    }).collect::<Vec<_>>()
                }
                Fields::Unnamed(ref fields) => {
                    fields.unnamed.iter().enumerate().map(|(i, f)| {
                        let ty = &f.ty;
                        quote! {
                            <#ty>::from_le_bytes(bytes[#i..#i+core::mem::size_of::<#ty>()].try_into().ok()?)
                        }
                    }).collect::<Vec<_>>()
                }
                Fields::Unit => vec![],
            };

            quote! {
                impl RBlueFromU8Array for #name {
                    fn from_u8_array(bytes: &[u8]) -> Option<Self> {
                        if bytes.len() != core::mem::size_of::<Self>() {
                            return None;
                        }
                        Some(#name {
                            #(#field_parsers),*
                        })
                    }
                }
            }
        }
        _ => unimplemented!(),
    };

    TokenStream::from(expanded)
}
