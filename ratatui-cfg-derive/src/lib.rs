use {
    proc_macro::TokenStream,
    quote::quote,
    syn::{Data, DeriveInput, Fields, GenericArgument, PathArguments, Type, parse_macro_input},
};

#[proc_macro_derive(ConfigMenu, attributes(config_menu))]
pub fn derive_config_menu(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let field_metadata = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let field_info = fields.named.iter().map(|f| {
                    let field_name = &f.ident;
                    let field_type = &f.ty;
                    let field_name_str = field_name.as_ref().unwrap().to_string();

                    let (is_nested, is_option, is_vec, inner_type, inner_type_ident) = analyze_type(field_type);

                    let (nested_getter, nested_metadata_getter, nested_setter) = if is_nested {
                        let inner_type_tokens = &inner_type_ident;
                        (
                            quote! {
                                Some(Box::new(|config: &dyn std::any::Any| -> Option<Box<dyn std::any::Any>> {
                                    config.downcast_ref::<#name>()
                                        .map(|c| Box::new(c.#field_name.clone()) as Box<dyn std::any::Any>)
                                }))
                            },
                            quote! {
                                Some(Box::new(|| {
                                    <#inner_type_tokens as ::config_menu::ConfigMenuTrait>::get_field_metadata()
                                }))
                            },
                            quote! {
                                Some(Box::new(|config: &mut dyn std::any::Any, value: Box<dyn std::any::Any>| -> Result<(), String> {
                                    if let Some(c) = config.downcast_mut::<#name>() {
                                        if let Some(nested) = value.downcast_ref::<#inner_type_tokens>() {
                                            c.#field_name = nested.clone();
                                            Ok(())
                                        } else {
                                            Err(format!("Type mismatch when setting nested field '{}'", #field_name_str))
                                        }
                                    } else {
                                        Err("Config type mismatch".to_string())
                                    }
                                }))
                            }
                        )
                    } else {
                        (quote! { None }, quote! { None }, quote! { None })
                    };

                    quote! {
                        ::config_menu::FieldMetadata {
                            name: #field_name_str,
                            is_nested: #is_nested,
                            is_option: #is_option,
                            is_vec: #is_vec,
                            field_type: ::config_menu::FieldType::from_str(#inner_type),
                            getter: Box::new(|config: &dyn std::any::Any| {
                                config.downcast_ref::<#name>()
                                    .map(|c| ::config_menu::format_field_value(&c.#field_name))
                            }),
                            setter: Box::new(|config: &mut dyn std::any::Any, value: String| {
                                if let Some(c) = config.downcast_mut::<#name>() {
                                    ::config_menu::parse_and_set(&mut c.#field_name, value)
                                } else {
                                    Err("Type mismatch".to_string())
                                }
                            }),
                            nested_getter: #nested_getter,
                            nested_metadata_getter: #nested_metadata_getter,
                            nested_setter: #nested_setter,
                        }
                    }
                });

                quote! {
                    vec![#(#field_info),*]
                }
            }
            _ => panic!("ConfigMenu only supports named fields"),
        },
        _ => panic!("ConfigMenu only supports structs"),
    };

    let generated = quote! {
        impl ::config_menu::ConfigMenuTrait for #name {
            fn get_field_metadata() -> Vec<::config_menu::FieldMetadata> {
                #field_metadata
            }

            fn get_menu_title() -> &'static str {
                stringify!(#name)
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
    };

    TokenStream::from(generated)
}

fn analyze_type(ty: &Type) -> (bool, bool, bool, String, Option<&syn::Ident>) {
    match ty {
        Type::Path(type_path) => {
            let last_segment = type_path.path.segments.last().unwrap();
            let ident = &last_segment.ident;
            let ident_str = ident.to_string();

            if ident_str == "Option"
                && let PathArguments::AngleBracketed(args) = &last_segment.arguments
                && let Some(GenericArgument::Type(inner)) = args.args.first()
            {
                let (nested, _, _, inner_type, inner_ident) = analyze_type(inner);
                return (nested, true, false, inner_type, inner_ident);
            }

            if ident_str == "Vec"
                && let PathArguments::AngleBracketed(args) = &last_segment.arguments
                && let Some(GenericArgument::Type(inner)) = args.args.first()
            {
                let (nested, _, _, inner_type, inner_ident) = analyze_type(inner);
                return (nested, false, true, inner_type, inner_ident);
            }

            let is_primitive = matches!(
                ident_str.as_str(),
                "bool"
                    | "i8"
                    | "i16"
                    | "i32"
                    | "i64"
                    | "i128"
                    | "u8"
                    | "u16"
                    | "u32"
                    | "u64"
                    | "u128"
                    | "f32"
                    | "f64"
                    | "String"
                    | "str"
                    | "usize"
                    | "isize"
            );

            if is_primitive {
                (false, false, false, ident_str, None)
            } else {
                (true, false, false, ident_str.clone(), Some(ident))
            }
        }
        _ => (false, false, false, "Unknown".to_string(), None),
    }
}
