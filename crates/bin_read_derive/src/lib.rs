extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Fields, Meta, parse_macro_input};

fn parse_int(s: &str) -> Result<u64, std::num::ParseIntError> {
    if let Some(hex) = s.strip_prefix("0x") {
        u64::from_str_radix(hex, 16)
    } else {
        s.parse::<u64>()
    }
}

fn parse_offset_attr(attrs: &[Attribute]) -> Option<u64> {
    for attr in attrs {
        if attr.path().is_ident("bin_read")
            && let Meta::List(meta_list) = &attr.meta
        {
            let token_str = meta_list.tokens.to_string();
            if token_str.starts_with("offset") {
                // Parse "offset = N"
                if let Some(eq_pos) = token_str.find('=') {
                    let offset_str = token_str[eq_pos + 1..].trim();
                    if let Ok(offset) = parse_int(offset_str) {
                        return Some(offset);
                    }
                }
            }
        }
    }
    None
}

#[proc_macro_derive(BinRead, attributes(bin_read))]
pub fn derive_bin_read(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => {
                let field_reads = fields.named.iter().map(|field| {
                    let field_name = &field.ident;
                    if let Some(offset) = parse_offset_attr(&field.attrs) {
                        quote! {
                            #field_name: {
                                reader.seek(std::io::SeekFrom::Start(position + #offset))?;
                                BinRead::bin_read(reader)?
                            }
                        }
                    } else {
                        quote! {
                            #field_name: BinRead::bin_read(reader)?
                        }
                    }
                });

                quote! {
                    impl #impl_generics BinRead for #name #ty_generics #where_clause {
                        fn bin_read<R: std::io::Read + std::io::Seek>(reader: &mut R) -> Result<Self, Box<dyn std::error::Error>> {
                            let position = reader.seek(std::io::SeekFrom::Current(0))?;
                            Ok(Self {
                                #(#field_reads,)*
                            })
                        }
                    }
                }
            }
            Fields::Unnamed(fields) => {
                let field_reads = fields.unnamed.iter().map(|_| {
                    quote! {
                        BinRead::bin_read(reader)?
                    }
                });

                quote! {
                    impl #impl_generics BinRead for #name #ty_generics #where_clause {
                        fn bin_read<R: std::io::Read + std::io::Seek>(reader: &mut R) -> Result<Self, Box<dyn std::error::Error>> {
                            Ok(Self(
                                #(#field_reads,)*
                            ))
                        }
                    }
                }
            }
            Fields::Unit => {
                quote! {
                    impl #impl_generics BinRead for #name #ty_generics #where_clause {
                        fn bin_read<R: std::io::Read + std::io::Seek>(_reader: &mut R) -> Result<Self, Box<dyn std::error::Error>> {
                            Ok(Self)
                        }
                    }
                }
            }
        },
        _ => {
            panic!("BinRead can only be derived for structs");
        }
    };

    TokenStream::from(expanded)
}
