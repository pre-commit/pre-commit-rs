use quote::quote;
use std::fs;

#[proc_macro_attribute]
pub fn make_config_hook(
    _attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let name = input.ident;

    let contents = fs::read_to_string("src/clientlib.rs").unwrap();
    let parsed = syn::parse_file(&contents).unwrap();

    let mut f_code: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut overlay_code: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut names: Vec<proc_macro2::TokenStream> = Vec::new();
    for item in parsed.items {
        if let syn::Item::Struct(syn::ItemStruct {
            ident,
            fields: syn::Fields::Named(fields),
            ..
        }) = item
        {
            if ident == "ManifestHook" {
                for field in fields.named {
                    let f_ident = field.ident.as_ref().unwrap();
                    let f_tp = field.ty;

                    names.push(quote! { #f_ident });

                    if f_ident == "id" {
                        f_code.push(quote! {
                            #[cfgv_id]
                            pub(crate) #f_ident: #f_tp
                        });
                    } else {
                        f_code.push(quote! {
                            pub(crate) #f_ident: Option<#f_tp>
                        });
                    }

                    if f_ident == "id" {
                        overlay_code.push(quote! {
                            let #f_ident = hook.#f_ident.clone();
                        });
                    } else {
                        overlay_code.push(quote! {
                            let #f_ident = match &self.#f_ident {
                                Some(val) => val.clone(),
                                None => hook.#f_ident.clone(),
                            };
                        });
                    }
                }
            }
        }
    }

    if f_code.is_empty() {
        panic!("expected ManifestHook to have fields");
    }

    let ret = quote! {
        #[derive(Cfgv, Debug)]
        pub(crate) struct #name {
            #(#f_code),*
        }

        impl #name {
            pub(crate) fn overlay_on(&self, hook: &ManifestHook) -> ManifestHook {
                #(#overlay_code)*
                ManifestHook { #(#names),* }
            }
        }
    };
    proc_macro::TokenStream::from(ret)
}

fn _pre_commit_env_vars(field: &syn::Field) -> Vec<String> {
    let mut ret: Vec<String> = Vec::new();
    for attr in &field.attrs {
        if let Some(ident) = attr.path().get_ident() {
            if ident == "pre_commit_env_var" {
                let s = attr.parse_args::<syn::LitStr>().unwrap();
                ret.push(s.value());
            }
        }
    }
    ret
}

fn _pre_commit_env_struct(
    name: &syn::Ident,
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> proc_macro2::TokenStream {
    let mut f_code = vec![quote! { std::env::set_var("PRE_COMMIT", "1"); }];
    for f in fields.iter() {
        let f_name = f.ident.as_ref().unwrap();
        for s in _pre_commit_env_vars(f) {
            f_code.push(quote! {
                if let Some(v) = &self.#f_name {
                    std::env::set_var(#s, v);
                }
            });
        }
    }
    quote! {
        impl PreCommitEnv for #name {
            fn set_pre_commit_env_vars(&self) {
                #(#f_code)*
            }
        }
    }
}

#[proc_macro_derive(PreCommitEnv, attributes(pre_commit_env_var))]
pub fn pre_commit_env(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    match input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(fields),
            ..
        }) => proc_macro::TokenStream::from(_pre_commit_env_struct(&input.ident, &fields.named)),
        _ => panic!("need struct with named fields"),
    }
}
