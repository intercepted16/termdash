//! Apply standard features to all level data; including serialization, reflection, etc for consistency.
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Attribute, Fields, Item, Meta, Path, Token, Type, parse::Parser, parse_macro_input,
    punctuated::Punctuated,
};

/// Inject reflection, standard derive features, and omit Option<T> values when None.
///
/// Provide comma-separated derive macros for additional ones.
#[proc_macro_attribute]
pub fn level_data(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item = parse_macro_input!(input as Item);

    let ident = match &item {
        Item::Struct(item_struct) => item_struct.ident.clone(),
        Item::Enum(item_enum) => item_enum.ident.clone(),
        _ => {
            return syn::Error::new_spanned(
                item,
                "#[level_data] can only be used on structs or enums",
            )
            .to_compile_error()
            .into();
        }
    };

    if let Item::Struct(ref mut item_struct) = item {
        modify_option_fields(&mut item_struct.fields);
    }

    let parser = Punctuated::<Path, Token![,]>::parse_terminated;
    let extra_derives = match parser.parse(args) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    let has_default = extra_derives.iter().any(|p| p.is_ident("Default"));

    let reflect_attr = if has_default {
        quote!(#[reflect(Serialize, Deserialize, Default)])
    } else {
        quote!(#[reflect(Serialize, Deserialize)])
    };

    quote! {
        #[derive(
            Clone,
            Debug,
            Deserialize,
            Reflect,
            Serialize,
            #extra_derives
        )]
        #reflect_attr
        #item

        inventory::submit! {
            crate::level::model::LevelDataRegistration(|app: &mut bevy::prelude::App| {
                app.register_type::<#ident>();
            })
        }
    }
    .into()
}
fn modify_option_fields(fields: &mut Fields) {
    for field in fields.iter_mut() {
        if is_option_type(&field.ty) {
            if !has_serde_attr(&field.attrs, "default") {
                let default_attr: Attribute = syn::parse_quote! { #[serde(default)] };
                field.attrs.push(default_attr);
            }

            if !has_serde_attr(&field.attrs, "skip_serializing_if") {
                let skip_attr: Attribute =
                    syn::parse_quote! { #[serde(skip_serializing_if = "Option::is_none")] };
                field.attrs.push(skip_attr);
            }
        }
    }
}

fn has_serde_attr(attrs: &[Attribute], name: &str) -> bool {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident("serde"))
        .any(|attr| {
            let Meta::List(list) = &attr.meta else {
                return false;
            };

            list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .is_ok_and(|nested| {
                    nested.iter().any(|meta| match meta {
                        Meta::Path(path) => path.is_ident(name),
                        Meta::NameValue(name_value) => name_value.path.is_ident(name),
                        Meta::List(list) => list.path.is_ident(name),
                    })
                })
        })
}

fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty
        && let Some(last_segment) = type_path.path.segments.last()
    {
        return last_segment.ident == "Option";
    }
    false
}
