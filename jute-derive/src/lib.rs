extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, GenericArgument};

fn remove_prefix(s: &String, prefix: &'static str) -> String {
    s.strip_prefix(prefix).unwrap_or(s).to_string()
}

fn get_generic_argument_str(x: &GenericArgument) -> syn::Result<String> {
    if let syn::GenericArgument::Type(t) = x {
        if let syn::Type::Path(tp) = t {
            return get_type_str(tp);
        }
    }
    return Err(syn::Error::new_spanned(x, "not supported".to_string()));
}

fn get_type_str(p: &syn::TypePath) -> syn::Result<String> {
    let mut segments = Vec::new();
    for segment in p.path.segments.iter() {
        // handle type names
        let mut n = segment.ident.to_string();

        // handle arguments, eg. <u8>
        match &segment.arguments {
            syn::PathArguments::None => {}
            syn::PathArguments::Parenthesized(_) => {
                return Err(syn::Error::new_spanned(p, "not supported".to_string()));
            }
            syn::PathArguments::AngleBracketed(a) => {
                let mut args = Vec::new();
                for x in a.args.iter() {
                    args.push(get_generic_argument_str(x)?);
                }
                n.push('<');
                n.push_str(args.join(", ").as_str());
                n.push('>');
            }
        }
        segments.push(n);
    }

    let mut name = segments.join("::");
    name = remove_prefix(&name, "std::string::");
    name = remove_prefix(&name, "string::");
    Ok(name)
}

fn expand(input: &DeriveInput) -> syn::Result<TokenStream> {
    if let syn::Data::Struct(s) = &input.data {
        let mut types = Vec::new();
        for field in s.fields.iter() {
            if let syn::Type::Path(p) = &field.ty {
                let type_str = get_type_str(&p)?;
                match type_str.as_str() {
                    // primitives
                    "bool" => types.push(quote!(jute_rs::FieldType::Bool)),
                    "i32" => types.push(quote!(jute_rs::FieldType::Integer)),
                    "i64" => types.push(quote!(jute_rs::FieldType::Long)),
                    "f32" => types.push(quote!(jute_rs::FieldType::Float)),
                    "f64" => types.push(quote!(jute_rs::FieldType::Double)),
                    "String" => types.push(quote!(jute_rs::FieldType::String)),

                    // Sequence
                    "Vec<u8>" => types.push(quote!(jute_rs::FieldType::Buffer)),
                    "Vec<bool>" => {
                        types.push(quote!(jute_rs::FieldType::Vector(Box::new(jute_rs::FieldType::Bool))))
                    }
                    "Vec<i32>" => types.push(quote!(jute_rs::FieldType::Vector(
                        Box::new(jute_rs::FieldType::Integer)
                    ))),
                    "Vec<i64>" => {
                        types.push(quote!(jute_rs::FieldType::Vector(Box::new(jute_rs::FieldType::Long))))
                    }
                    "Vec<f32>" => types.push(quote!(jute_rs::FieldType::Vector(
                        Box::new(jute_rs::FieldType::Float)
                    ))),
                    "Vec<f64>" => types.push(quote!(jute_rs::FieldType::Vector(
                        Box::new(jute_rs::FieldType::Double)
                    ))),
                    "Vec<String>" => types.push(quote!(jute_rs::FieldType::Vector(
                        Box::new(jute_rs::FieldType::String)
                    ))),
                    "Vec<Vec<u8>>" => types.push(quote!(jute_rs::FieldType::Vector(
                        Box::new(jute_rs::FieldType::Buffer)
                    ))),

                    // Map
                    "HashMap<String, bool>" => {
                        types.push(quote!(jute_rs::FieldType::Map(Box::new(jute_rs::FieldType::Bool))))
                    }
                    "HashMap<String, i32>" => {
                        types.push(quote!(jute_rs::FieldType::Map(Box::new(jute_rs::FieldType::Integer))))
                    }
                    "HashMap<String, i64>" => {
                        types.push(quote!(jute_rs::FieldType::Map(Box::new(jute_rs::FieldType::Long))))
                    }
                    "HashMap<String, f32>" => {
                        types.push(quote!(jute_rs::FieldType::Map(Box::new(jute_rs::FieldType::Float))))
                    }
                    "HashMap<String, f64>" => {
                        types.push(quote!(jute_rs::FieldType::Map(Box::new(jute_rs::FieldType::Double))))
                    }
                    "HashMap<String, String>" => {
                        types.push(quote!(jute_rs::FieldType::Map(Box::new(jute_rs::FieldType::String))))
                    }
                    "HashMap<String, Vec<u8>>" => {
                        types.push(quote!(jute_rs::FieldType::Map(Box::new(jute_rs::FieldType::Buffer))))
                    }
                    "HashMap<String, Vec<bool>>" => types.push(quote!(jute_rs::FieldType::Map(
                        Box::new(jute_rs::FieldType::Vector(Box::new(jute_rs::FieldType::Bool)))
                    ))),
                    "HashMap<String, Vec<i32>>" => types.push(quote!(jute_rs::FieldType::Map(
                        Box::new(jute_rs::FieldType::Vector(Box::new(jute_rs::FieldType::Integer)))
                    ))),
                    "HashMap<String, Vec<i64>>" => types.push(quote!(jute_rs::FieldType::Map(
                        Box::new(jute_rs::FieldType::Vector(Box::new(jute_rs::FieldType::Long)))
                    ))),
                    "HashMap<String, Vec<f32>>" => types.push(quote!(jute_rs::FieldType::Map(
                        Box::new(jute_rs::FieldType::Vector(Box::new(jute_rs::FieldType::Float)))
                    ))),
                    "HashMap<String, Vec<f64>>" => types.push(quote!(jute_rs::FieldType::Map(
                        Box::new(jute_rs::FieldType::Vector(Box::new(jute_rs::FieldType::Double)))
                    ))),
                    "HashMap<String, Vec<String>>" => types.push(quote!(jute_rs::FieldType::Map(
                        Box::new(jute_rs::FieldType::Vector(Box::new(jute_rs::FieldType::String)))
                    ))),
                    "HashMap<String, Vec<Vec<u8>>>" => types.push(quote!(jute_rs::FieldType::Map(
                        Box::new(jute_rs::FieldType::Vector(Box::new(jute_rs::FieldType::Buffer)))
                    ))),

                    // Unknown
                    _ => {
                        return Err(syn::Error::new_spanned(
                            input,
                            format!("Unexpected type: {}", type_str),
                        ));
                    }
                }
            }
        }

        let name_ident = &input.ident;
        let name = &name_ident.to_string();

        let gen = quote! {
            impl jute_rs::Jute for #name_ident {
                fn type_name() -> &'static str {
                    #name
                }

                fn field_types() -> Vec<jute_rs::FieldType> {
                    vec![#(#types),*]
                }
            }
        };
        Ok(gen.into())
    } else {
        Err(syn::Error::new_spanned(
            input,
            "Must define on a Struct, not Enum".to_string(),
        ))
    }
}

#[proc_macro_derive(Jute)]
pub fn derive_jute(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    match expand(&input) {
        Ok(s) => s.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
