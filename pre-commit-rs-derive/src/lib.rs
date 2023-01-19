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
    for item in parsed.items {
        match item {
            syn::Item::Struct(syn::ItemStruct {
                ident,
                fields: syn::Fields::Named(fields),
                ..
            }) => {
                if ident == "ManifestHook" {
                    for field in fields.named {
                        let f_ident = field.ident.as_ref().unwrap();
                        let f_tp = field.ty;
                        if f_ident == "id" {
                            f_code.push(quote::quote! {
                                #[cfgv_id]
                                #f_ident: #f_tp
                            });
                        } else {
                            f_code.push(quote::quote! {
                                #f_ident: Option<#f_tp>
                            });
                        }
                    }
                }
            }
            _ => (),
        }
    }

    if f_code.len() == 0 {
        panic!("expected ManifestHook to have fields");
    }

    let ret = quote::quote! {
        #[derive(Cfgv, Debug)]
        struct #name {
            #(#f_code),*
        }
    };
    proc_macro::TokenStream::from(ret)
}
