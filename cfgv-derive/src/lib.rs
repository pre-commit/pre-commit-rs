use quote::quote;

fn _unpack_option(t: &syn::Type) -> Option<&syn::Type> {
    match t {
        syn::Type::Path(syn::TypePath {
            path: syn::Path { segments, .. },
            ..
        }) => {
            if segments.len() != 1 {
                return None;
            }

            match &segments[0] {
                syn::PathSegment {
                    ident,
                    arguments:
                        syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                            args,
                            ..
                        }),
                } => {
                    if ident != "Option" || args.len() != 1 {
                        return None;
                    }

                    match &args[0] {
                        syn::GenericArgument::Type(t) => Some(&t),
                        _ => None,
                    }
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn _get_attr<'a>(f: &'a syn::Field, name: &str) -> Option<&'a syn::Attribute> {
    for attr in &f.attrs {
        match attr.path.get_ident() {
            Some(ident) => {
                if ident == name {
                    return Some(attr);
                }
            }
            _ => (),
        }
    }
    None
}

fn _struct(
    name: &syn::Ident,
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> proc_macro2::TokenStream {
    let mut id: Option<(&syn::Ident, proc_macro2::TokenStream)> = None;
    let mut f_names: Vec<&syn::Ident> = Vec::new();
    let mut f_code: Vec<proc_macro2::TokenStream> = Vec::new();
    for f in fields.iter() {
        let f_name = f.ident.as_ref().unwrap();
        let f_tp = &f.ty;
        let ctx_str = format!("At key: {f_name}");
        let key_str = format!("{f_name}");

        let code = if let Some(f_tp) = _unpack_option(f_tp) {
            quote! {
                ctx.push(#ctx_str.into());
                let #f_name = match v.get(#key_str) {
                    Some(val) => Some(<#f_tp>::cfgv_validate(ctx, val)?),
                    None => None,
                };
                ctx.pop();
            }
        } else if _get_attr(f, "cfgv_default").is_some() {
            quote! {
                ctx.push(#ctx_str.into());
                let #f_name = match v.get(#key_str) {
                    Some(val) => <#f_tp>::cfgv_validate(ctx, val)?,
                    None => <#f_tp>::default(),
                };
                ctx.pop();
            }
        } else if let Some(attr) = _get_attr(f, "cfgv_default_expr") {
            let expr = attr.parse_args::<syn::Expr>().unwrap();
            quote! {
                ctx.push(#ctx_str.into());
                let #f_name = match v.get(#key_str) {
                    Some(val) => <#f_tp>::cfgv_validate(ctx, val)?,
                    None => { #expr },
                };
                ctx.pop();
            }
        } else {
            let missing_key_msg = format!("Missing required key: {f_name}");
            quote! {
                ctx.push(#ctx_str.into());
                let #f_name = match v.get(#key_str) {
                    Some(val) => <#f_tp>::cfgv_validate(ctx, val)?,
                    None => anyhow::bail!(cfgv::ctx_s(ctx, #missing_key_msg)),
                };
                ctx.pop();
            }
        };

        f_names.push(f_name);

        if _get_attr(f, "cfgv_id").is_some() {
            id = Some((f_name, code));
        } else {
            f_code.push(code);
        }
    }

    let id_code = if let Some((f_name, code)) = id {
        let ctx_str_1 = format!("At {name}({f_name}=MISSING)");
        let ctx_str_2 = format!("At {name}({f_name}={{}})");
        quote! {
            ctx.push(#ctx_str_1.into());
            #code
            ctx.pop();
            ctx.push(format!(#ctx_str_2, #f_name).into());
        }
    } else {
        let ctx_str = format!("At {name}()");
        quote! { ctx.push(#ctx_str.into()); }
    };

    let expected_map = format!("Expected a {name} map but got a {{}}");
    quote! {
        impl Cfgv for #name {
            fn cfgv_validate(ctx: &mut Vec<String>, v: &serde_yaml::Value) -> anyhow::Result<Self> {
                if let serde_yaml::Value::Mapping(v) = v {
                    #id_code
                    #(#f_code)*
                    ctx.pop();
                    Ok(#name { #(#f_names),* })
                } else {
                    anyhow::bail!(#expected_map, cfgv::type_name(v));
                }
            }
        }
    }
}

#[proc_macro_derive(Cfgv, attributes(cfgv_id, cfgv_default, cfgv_default_expr))]
pub fn cfgv(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(fields),
            ..
        }) => proc_macro::TokenStream::from(_struct(&input.ident, &fields.named)),
        _ => panic!("need struct with named fields"),
    }
}
