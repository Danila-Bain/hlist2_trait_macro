use proc_macro;
use quote;
use syn;

mod angle_bracketed_generic_params;
use angle_bracketed_generic_params::AngleBracketedGenericParams;

#[allow(non_snake_case)]
#[proc_macro]
pub fn TraitHList(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as TraitHListInput);
    proc_macro::TokenStream::from(input.expand())
}

struct TraitHListInput {
    hlist_trait: syn::Ident,
    base_trait: syn::Ident,
    trait_generic_params: Vec<syn::GenericParam>,
    trait_where_clause: Option<syn::WhereClause>,
    methods: Vec<TraitHListMethod>,
}

#[derive(Clone)]
struct TraitHListMethod {
    item_fn_name: syn::Ident,
    hlist_fn_name: syn::Ident,
    hlist_fn_name_all: Option<syn::Ident>,
    hlist_fn_name_any: Option<syn::Ident>,

    sig: syn::Signature,

    item_output: syn::Type,
    hlist_output_ident: syn::Ident,

    args: proc_macro2::TokenStream,
    args_cloned: proc_macro2::TokenStream,
}

impl TraitHListMethod {
    fn new(
        syn::TraitItemFn {
            attrs,
            sig,
            default,
            semi_token: _,
        }: syn::TraitItemFn,
    ) -> Self {
        {
            let item_fn_name = sig.ident.clone();

            assert!(
                default.is_none(),
                "Default implementation is not supported in methods."
            );

            let hlist_fn_name = match attrs.len() {
                0 => item_fn_name.clone(),
                1 => match &attrs[0] {
                    syn::Attribute {
                        meta:
                            syn::Meta::NameValue(syn::MetaNameValue {
                                path: lhs,
                                eq_token: _,
                                value:
                                    syn::Expr::Path(syn::ExprPath {
                                        attrs,
                                        qself: None,
                                        path: rhs,
                                    }),
                            }),
                        ..
                    } if attrs.is_empty() && lhs.is_ident("name") => {
                        if let Some(name) = rhs.get_ident() {
                            name.clone()
                        } else {
                            panic!("Name must be a simple identifier without path.")
                        }
                    }
                    _ => panic!(
                        "Unsupported method attribute. Try #[name = <other_method_name>] without quotes."
                    ),
                },
                _ => panic!("Multiple method attributes are not supported."),
            };

            let item_output: syn::Type = match sig.output.clone() {
                syn::ReturnType::Default => syn::parse_quote!(()),
                syn::ReturnType::Type(_, ty) => *ty,
            };

            let hlist_output_ident =
                quote::format_ident!("{}HListOutput", sig.ident.to_string().to_uppercase());

            let sig = syn::Signature {
                output: syn::parse_quote! { -> Self::#hlist_output_ident},
                ..sig.clone()
            };

            let (hlist_fn_name_all, hlist_fn_name_any) = if let syn::Type::Path(ref ty) =
                item_output
                && ty.qself.is_none()
                && ty.path.is_ident("bool")
            {
                (
                    Some(quote::format_ident!("all_{}", hlist_fn_name)),
                    Some(quote::format_ident!("any_{}", hlist_fn_name)),
                )
            } else {
                (None, None)
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
            let args = quote::quote! {#(#args),*};
            let args_cloned = quote::quote! {#(#args_cloned),*};

            Self {
                item_fn_name,
                hlist_fn_name,
                hlist_fn_name_all,
                hlist_fn_name_any,
                sig,
                item_output,
                hlist_output_ident,
                args,
                args_cloned,
            }
        }
    }
}

impl syn::parse::Parse for TraitHListInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let hlist_trait: syn::Ident = input.parse()?;
        input.parse::<syn::Token![for]>()?;
        input.parse::<syn::Token![trait]>()?;
        let base_trait: syn::Ident = input.parse()?;

        let trait_generic_params = if input.peek(syn::Token![<]) {
            let bracketed: AngleBracketedGenericParams = input.parse()?;
            bracketed.params.into_iter().collect()
        } else {
            vec![]
        };

        let trait_where_clause: Option<syn::WhereClause> = if input.peek(syn::Token![where]) {
            Some(input.parse()?)
        } else {
            None
        };

        let inner;
        syn::braced!(inner in input);
        let mut methods = Vec::new();
        while !inner.is_empty() {
            match inner.parse()? {
                syn::TraitItem::Fn(trait_item_fn) => {
                    methods.push(TraitHListMethod::new(trait_item_fn));
                }
                syn::TraitItem::Const(_trait_item_const) => {
                    panic!("Const items in traits are not supported.")
                }
                syn::TraitItem::Type(_trait_item_type) => {
                    panic!("Type items in traits are not supported.")
                }
                syn::TraitItem::Macro(_trait_item_macro) => {
                    panic!("Macro items in traits are not supported.")
                }
                syn::TraitItem::Verbatim(_token_stream) => {
                    panic!("Extra tokens in traits are not supported.")
                }
                _ => panic!("Unsupported item in trait."),
            }
        }

        Ok(Self {
            hlist_trait,
            base_trait,
            methods,
            trait_generic_params,
            trait_where_clause,
        })
    }
}

impl TraitHListInput {
    fn expand(&self) -> proc_macro2::TokenStream {
        let Self {
            hlist_trait,
            base_trait,
            methods,
            trait_generic_params,
            trait_where_clause,
        } = self;

        let method_defs = methods.iter().map(
            |TraitHListMethod {
                 hlist_output_ident: output,
                 hlist_fn_name,
                 sig,
                 ..
             }| {
                let sig = syn::Signature {
                    ident: hlist_fn_name.clone(),
                    ..sig.clone()
                };
                quote::quote! { type #output; #sig; }
            },
        );

        let bool_method_defs = methods.iter().filter_map(
            |TraitHListMethod {
                 sig,
                 hlist_fn_name_all,
                 hlist_fn_name_any,
                 ..
             }| {
                let (Some(name_all), Some(name_any)) = (hlist_fn_name_all, hlist_fn_name_any)
                else {
                    return None;
                };
                let sig_all = syn::Signature {
                    output: syn::parse_quote! {-> bool},
                    ident: name_all.clone(),
                    ..sig.clone()
                };
                let sig_any = syn::Signature {
                    output: syn::parse_quote! {-> bool},
                    ident: name_any.clone(),
                    ..sig.clone()
                };
                Some(quote::quote! {
                    #sig_all;
                    #sig_any;
                })
            },
        );

        let allow_unused_variables: syn::Attribute = syn::parse_quote!(#[allow(unused_variables)]);
        let nil_impls = methods.iter().map(
            |TraitHListMethod {
                 sig,
                 hlist_fn_name,
                 hlist_output_ident: hlist_fn_output,
                 ..
             }| {
                let sig = syn::Signature {
                    ident: hlist_fn_name.clone(),
                    ..sig.clone()
                };
                quote::quote! {
                    type #hlist_fn_output = hlist2::Nil;
                    #sig { hlist2::Nil }
                }
            },
        );
        let nil_bool_impls = methods.iter().filter_map(
            |TraitHListMethod {
                 sig,
                 hlist_fn_name_all,
                 hlist_fn_name_any,
                 ..
             }| {
                let (Some(name_all), Some(name_any)) = (hlist_fn_name_all, hlist_fn_name_any)
                else {
                    return None;
                };
                let sig_all = syn::Signature {
                    output: syn::parse_quote! {-> bool},
                    ident: name_all.clone(),
                    ..sig.clone()
                };
                let sig_any = syn::Signature {
                    output: syn::parse_quote! {-> bool},
                    ident: name_any.clone(),
                    ..sig.clone()
                };
                Some(quote::quote! {
                    #sig_all {true}
                    #sig_any {false}
                })
            },
        );

        let cons_impls = methods.iter().map(
            |TraitHListMethod {
                 sig,
                 item_fn_name,
                 hlist_fn_name,
                 item_output,
                 hlist_output_ident,
                 args,
                 args_cloned,
                 ..
             }| {
                 let sig = syn::Signature { ident: hlist_fn_name.clone(), ..sig.clone()};
                 quote::quote! {
                     type #hlist_output_ident = hlist2::Cons<#item_output, __HListTail::#hlist_output_ident>;
                     #sig {
                         let hlist2::Cons(__hlist_head, __hlist_tail) = self;
                         hlist2::Cons(__hlist_head.#item_fn_name(#args_cloned), __hlist_tail.#hlist_fn_name(#args))
                     }
                 }
            },
        );
        let cons_bool_impls = methods.iter().filter_map(
            |TraitHListMethod {
                 sig,
                 item_fn_name,
                 hlist_fn_name_all,
                 hlist_fn_name_any,
                 args,
                 args_cloned,
                 ..
             }| {
                let (Some(name_all), Some(name_any)) = (hlist_fn_name_all, hlist_fn_name_any)
                else {
                    return None;
                };
                let sig_all = syn::Signature {
                    output: syn::parse_quote! {-> bool},
                    ident: name_all.clone(),
                    ..sig.clone()
                };
                let sig_any = syn::Signature {
                    output: syn::parse_quote! {-> bool},
                    ident: name_any.clone(),
                    ..sig.clone()
                };

                Some(quote::quote! {
                    #sig_all {
                        let hlist2::Cons(__hlist_head, __hlist_tail) = self;
                        __hlist_head.#item_fn_name(#args_cloned) && __hlist_tail.#name_all(#args)
                    }
                    #sig_any {
                        let hlist2::Cons(__hlist_head, __hlist_tail) = self;
                        __hlist_head.#item_fn_name(#args_cloned) || __hlist_tail.#name_any(#args)
                    }
                })
            },
        );

        let trait_generic_args: Vec<_> = trait_generic_params
            .iter()
            .map(|param| match param {
                syn::GenericParam::Lifetime(syn::LifetimeParam { lifetime, .. }) => {
                    quote::quote!(#lifetime)
                }
                syn::GenericParam::Type(syn::TypeParam { ident, .. }) => quote::quote!(#ident),
                syn::GenericParam::Const(syn::ConstParam { ident, .. }) => {
                    quote::quote!(#ident)
                }
            })
            .collect();

        quote::quote! {
            trait #hlist_trait<#(#trait_generic_params),*> #trait_where_clause {
                #(#method_defs)*
                #(#bool_method_defs)*
            }

            #allow_unused_variables
            impl<#(#trait_generic_params),*>
                #hlist_trait<#(#trait_generic_args),*> for hlist2::Nil #trait_where_clause {
                #(#nil_impls)*
                #(#nil_bool_impls)*
            }
            //
            impl<
                #(#trait_generic_params,)*
                __HListHead: #base_trait <#(#trait_generic_args),*>,
                __HListTail: #hlist_trait<#(#trait_generic_args),*>
            > #hlist_trait<#(#trait_generic_args),*> for hlist2::Cons<__HListHead, __HListTail> #trait_where_clause {
                #(#cons_impls)*
                #(#cons_bool_impls)*
            }
        }
    }
}
