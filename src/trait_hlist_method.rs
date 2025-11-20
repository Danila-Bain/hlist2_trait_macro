#[derive(Clone)]
pub struct TraitHListMethod {
    pub item_fn_ident: syn::Ident,
    pub hlist_fn_ident: syn::Ident,
    pub hlist_fn_ident_at_index: syn::Ident,
    pub hlist_fn_ident_all: Option<syn::Ident>,
    pub hlist_fn_ident_any: Option<syn::Ident>,

    pub hlist_fn_sig: syn::Signature,
    pub hlist_fn_sig_at_index: syn::Signature,
    pub hlist_fn_sig_all: Option<syn::Signature>,
    pub hlist_fn_sig_any: Option<syn::Signature>,

    pub item_output: syn::Type,
    pub item_generic_params: Vec<syn::GenericParam>,
    pub item_generic_args: Vec<syn::GenericArgument>,
    pub item_where_clause: Option<syn::WhereClause>,
    pub hlist_output_ident: syn::Ident,

    pub args: Vec<proc_macro2::TokenStream>,
    pub args_cloned: Vec<proc_macro2::TokenStream>,
}

impl TraitHListMethod {
    pub fn new(
        syn::TraitItemFn {
            attrs,
            sig,
            default,
            semi_token: _,
        }: syn::TraitItemFn,
    ) -> Self {
        {
            assert!(
                default.is_none(),
                "Default implementation is not supported in methods."
            );

            let item_fn_ident = sig.ident.clone();

            let mut hlist_fn_ident = item_fn_ident.clone();

            for attr in attrs {
                match &attr {
                    syn::Attribute {
                        meta:
                            syn::Meta::NameValue(syn::MetaNameValue {
                                path: key,
                                eq_token: _,
                                value:
                                    syn::Expr::Path(syn::ExprPath {
                                        attrs: attrs2,
                                        qself: None,
                                        path: value,
                                    }),
                            }),
                        ..
                    } if attrs2.is_empty() && key.is_ident("name") => {
                        if let Some(name) = value.get_ident() {
                            hlist_fn_ident = name.clone()
                        } else {
                            panic!("Name must be a simple identifier without path.")
                        }
                    }
                    _ => panic!(
                        "Unsupported method attribute or format. Try #[name = <other_method_name>] without quotes."
                    ),
                }
            }
            let hlist_fn_ident_at_index = quote::format_ident!("{}_at_index", hlist_fn_ident);

            let item_output: syn::Type = match sig.output.clone() {
                syn::ReturnType::Default => syn::parse_quote!(()),
                syn::ReturnType::Type(_, ty) => *ty,
            };

            let hlist_output_ident =
                quote::format_ident!("{}HListOutput", sig.ident.to_string().to_uppercase());

            let item_generic_params: Vec<syn::GenericParam> =
                sig.generics.params.clone().into_iter().collect();
            let item_where_clause = sig.generics.where_clause.clone();

            let item_generic_args: Vec<syn::GenericArgument> = item_generic_params
                .clone()
                .into_iter()
                .map(crate::generic_param_to_arg::generic_param_to_arg)
                .collect();

            let mut hlist_fn_ident_all = None;
            let mut hlist_fn_ident_any = None;
            let mut hlist_fn_sig_all = None;
            let mut hlist_fn_sig_any = None;

            if let syn::Type::Path(ref ty) = item_output
                && ty.qself.is_none()
                && ty.path.is_ident("bool")
            {
                hlist_fn_ident_all = Some(quote::format_ident!("all_{}", hlist_fn_ident));
                hlist_fn_ident_any = Some(quote::format_ident!("any_{}", hlist_fn_ident));
                hlist_fn_sig_all = Some(syn::Signature {
                    output: syn::parse_quote! {-> bool},
                    ident: hlist_fn_ident_all.clone().unwrap(),
                    ..sig.clone()
                });
                hlist_fn_sig_any = Some(syn::Signature {
                    output: syn::parse_quote! {-> bool},
                    ident: hlist_fn_ident_any.clone().unwrap(),
                    ..sig.clone()
                });
            };

            let hlist_fn_sig = syn::Signature {
                output: syn::parse_quote! { -> Self::#hlist_output_ident<#(#item_generic_args),*>},
                ident: hlist_fn_ident.clone(),
                ..sig.clone()
            };

            let hlist_fn_sig_at_index = syn::Signature {
                ident: hlist_fn_ident_at_index.clone(),
                inputs: {
                    let mut inputs = sig.inputs.clone();
                    inputs.push(syn::parse_quote!(__hlist_index: usize));
                    inputs
                },
                ..sig.clone()
            };

            let mut args = vec![];
            let mut args_cloned = vec![];
            for fn_arg in sig.inputs.iter() {
                match fn_arg {
                    syn::FnArg::Receiver(_) => {}
                    syn::FnArg::Typed(syn::PatType { pat, ty, .. }) => match &**pat {
                        syn::Pat::Ident(ident) => {
                            args.push(quote::quote!(#ident));
                            match **ty {
                                syn::Type::Reference(_) => args_cloned.push(quote::quote!(#ident)),
                                _ => args_cloned.push(quote::quote!(Clone::clone(&#ident))),
                            }
                        }
                        _ => panic!(
                            "Unsupported argument pattern in function '{}': only simple identifiers are supported",
                            sig.ident
                        ),
                    },
                }
            }

            Self {
                item_fn_ident,
                hlist_fn_ident,
                hlist_fn_ident_at_index,
                hlist_fn_ident_all,
                hlist_fn_ident_any,
                hlist_fn_sig,
                hlist_fn_sig_at_index,
                hlist_fn_sig_all,
                hlist_fn_sig_any,
                item_generic_params,
                item_generic_args,
                item_where_clause,
                item_output,
                hlist_output_ident,
                args,
                args_cloned,
            }
        }
    }
}
