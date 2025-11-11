use proc_macro;
use quote;
use syn;

#[allow(non_snake_case)]
#[proc_macro]
pub fn TraitHList(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as HListTraitInput);
    proc_macro::TokenStream::from(input.expand())
}

struct HListTraitInput {
    hlist_trait: syn::Ident,
    base_trait: syn::Ident,
    trait_generic_params: Vec<syn::GenericParam>,
    trait_where_clause: Option<syn::WhereClause>,
    fns: Vec<syn::Signature>,
}

impl syn::parse::Parse for HListTraitInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let hlist_trait: syn::Ident = input.parse()?;
        input.parse::<syn::Token![for]>()?;
        input.parse::<syn::Token![trait]>()?;
        let base_trait: syn::Ident = input.parse()?;
        let mut trait_generic_params = vec![];

        if input.peek(syn::Token![<]) {
            input.parse::<syn::Token![<]>()?;
            loop {
                if input.peek(syn::Token![>]) {
                    input.parse::<syn::Token![>]>()?;
                    break
                } else {
                   trait_generic_params.push(input.parse()?);
                   if input.peek(syn::Token![,]) {
                       input.parse::<syn::Token![,]>()?;
                   }
                }
            }
        };
        let trait_where_clause: Option<syn::WhereClause> = if input.peek(syn::Token![where]) {
            Some(input.parse()?)
        } else {
            None
        };

        let inner;
        syn::braced!(inner in input);
        let mut fns = Vec::new();
        while !inner.is_empty() {
            match inner.parse()? {
                syn::TraitItem::Fn(syn::TraitItemFn {attrs, sig, default, ..}) => {
                    fns.push(sig);
                    assert!(default.is_none(), "Default implementation is not supported."); 
                    assert!(attrs.is_empty(), "Attributes for methods are not supported."); 
                }
                syn::TraitItem::Const(_trait_item_const) => panic!("Const items in traits are not supported."),
                syn::TraitItem::Type(_trait_item_type) => panic!("Type items in traits are not supported."),
                syn::TraitItem::Macro(_trait_item_macro) => panic!("Macro items in traits are not supported."),
                syn::TraitItem::Verbatim(_token_stream) => panic!("Extra tokens in traits are not supported."),
                _ => panic!("Unsupported item in trait."),
            }
        }

        Ok(Self {
            hlist_trait,
            base_trait,
            fns,
            trait_generic_params,
            trait_where_clause,
        })
    }
}

impl HListTraitInput {
    fn expand(&self) -> proc_macro2::TokenStream {
        let Self {
            hlist_trait,
            base_trait,
            fns,
            trait_generic_params: trait_generic_arg_defs,
            trait_where_clause,
        } = self;

        #[derive(Clone)]
        struct ParsedSignature {
            sig: syn::Signature,
            fn_output: proc_macro2::TokenStream,
            hlist_fn_output: syn::Ident,
            name: syn::Ident,
            name_all: syn::Ident,
            name_any: syn::Ident,

            call: proc_macro2::TokenStream,
            call_cloned: proc_macro2::TokenStream,
            is_bool: bool,
        }

        let parsed_fns : Vec<ParsedSignature> = fns.iter().map(|sig| {
            let output = match &sig.output {
                syn::ReturnType::Default => quote::quote!(()),
                syn::ReturnType::Type(_, ty) => quote::quote!(#ty),
            };
            let output_hlist = quote::format_ident!("{}HListOutput", sig.ident.to_string().to_uppercase());

            let sig_hlist = syn::Signature {
                output: syn::parse_quote! { -> Self::#output_hlist},
                ..sig.clone()
            };
            let is_bool = matches!(
                &sig.output,
                syn::ReturnType::Type(_, ty)
                    if matches!(&**ty, syn::Type::Path(tp)
                        if tp.qself.is_none() && tp.path.is_ident("bool"))
            );
            let args : Vec<proc_macro2::TokenStream> = sig.inputs.iter().filter_map(|fn_arg| match fn_arg {
                syn::FnArg::Receiver(_) => None, 
                syn::FnArg::Typed(pat_type) => match &*pat_type.pat {
                    syn::Pat::Ident(ident) => Some(quote::quote!(#ident)),
                    _ => panic!(
                        "Unsupported argument pattern in function '{}': only simple identifiers are supported",
                        sig.ident
                    ),
                },
            }).collect();

            let name = sig.ident.clone();
            let call = quote::quote!{#(#args),*};
            let call_cloned = quote::quote!{#(Clone::clone(&#args)),*};

            ParsedSignature {
                name_all: quote::format_ident!("all_{}", name.clone()),
                name_any: quote::format_ident!("any_{}", name.clone()),
                name,
                sig: sig_hlist,
                fn_output: output,
                hlist_fn_output: output_hlist,
                is_bool,
                call,
                call_cloned,
            }
        }).collect();

        let parsed_bool_fns : Vec<ParsedSignature> = parsed_fns.clone().into_iter().filter(|f| f.is_bool).collect();

        let fn_defs = parsed_fns
            .iter()
            .map(|ParsedSignature { hlist_fn_output: output, sig, .. }| {
                quote::quote! { type #output; #sig; }
            });
        let fn_bool_defs = parsed_bool_fns.iter().map(|ParsedSignature {sig, name_all, name_any, ..}| {
            let sig_all = syn::Signature { output: syn::parse_quote!{-> bool}, ident: name_all.clone(), ..sig.clone()};
            let sig_any = syn::Signature { output: syn::parse_quote!{-> bool}, ident: name_any.clone(), ..sig.clone()};
            quote::quote! {
                #sig_all;
                #sig_any;
            }
        });

        let allow_unused_variables: syn::Attribute = syn::parse_quote!(#[allow(unused_variables)]);
        let nil_impls = parsed_fns.iter().map(|ParsedSignature { sig, hlist_fn_output, .. }| {
            quote::quote! { 
                type #hlist_fn_output = hlist2::Nil; 
                #sig { hlist2::Nil } 
            }
        });
        let nil_bool_impls = parsed_bool_fns.iter().map(|ParsedSignature {sig, name_all, name_any, ..}| {
            let sig_all = syn::Signature { output: syn::parse_quote!{-> bool}, ident: name_all.clone(), ..sig.clone()};
            let sig_any = syn::Signature { output: syn::parse_quote!{-> bool}, ident: name_any.clone(), ..sig.clone()};
            quote::quote! {
                #sig_all {true}
                #sig_any {false}
            }
        });

        let cons_impls = parsed_fns.iter().map(|ParsedSignature { sig, name, fn_output, hlist_fn_output, call, call_cloned, .. }| {
            quote::quote! {
                type #hlist_fn_output = hlist2::Cons<#fn_output, __HListTail::#hlist_fn_output>;
                #sig {
                    let hlist2::Cons(__hlist_head, __hlist_tail) = self;
                    hlist2::Cons(__hlist_head.#name(#call_cloned), __hlist_tail.#name(#call))
                }
            }
        });
        let cons_bool_impls = parsed_bool_fns.iter().map(|ParsedSignature {sig, name, name_all, name_any, call, call_cloned, ..}| {
            let sig_all = syn::Signature { output: syn::parse_quote!{-> bool}, ident: name_all.clone(), ..sig.clone()};
            let sig_any = syn::Signature { output: syn::parse_quote!{-> bool}, ident: name_any.clone(), ..sig.clone()};
 
            quote::quote! {
                #sig_all {
                    let hlist2::Cons(__hlist_head, __hlist_tail) = self;
                    __hlist_head.#name(#call_cloned) && __hlist_tail.#name_all(#call)
                }
                #sig_any {
                    let hlist2::Cons(__hlist_head, __hlist_tail) = self;
                    __hlist_head.#name(#call_cloned) || __hlist_tail.#name_any(#call)
                }
            }
        });

        let trait_generic_args = trait_generic_arg_defs.iter().map(|arg| match arg {
            syn::GenericParam::Lifetime(syn::LifetimeParam{ lifetime, .. }) => quote::quote!(#lifetime),
            syn::GenericParam::Type(syn::TypeParam{ ident, .. }) => quote::quote!(#ident),
            syn::GenericParam::Const(syn::ConstParam { ident, .. }) => quote::quote!(#ident),
        });
        let trait_generic_args = quote::quote!(#(#trait_generic_args),*);

        quote::quote! {
            trait #hlist_trait<#(#trait_generic_arg_defs),*> #trait_where_clause {
                #(#fn_defs)*
                #(#fn_bool_defs)*
            }

            #allow_unused_variables
            impl<#(#trait_generic_arg_defs, )*>
                #hlist_trait<#trait_generic_args> for hlist2::Nil #trait_where_clause {
                #(#nil_impls)*
                #(#nil_bool_impls)*
            }
            //
            impl<
                #(#trait_generic_arg_defs, )*
                __HListHead: #base_trait <#trait_generic_args>, 
                __HListTail: #hlist_trait<#trait_generic_args>
            > #hlist_trait<#trait_generic_args> for hlist2::Cons<__HListHead, __HListTail> #trait_where_clause {
                #(#cons_impls)*
                #(#cons_bool_impls)*
            }
        }
    }
}
