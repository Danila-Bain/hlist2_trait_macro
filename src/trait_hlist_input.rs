pub struct TraitHListInput {
    pub vis: Option<syn::Token![pub]>,
    pub hlist_trait: syn::Ident,
    pub base_trait: syn::Ident,
    pub trait_generic_params: Vec<syn::GenericParam>,
    pub trait_where_clause: Option<syn::WhereClause>,
    pub methods: Vec<crate::TraitHListMethod>,
}

impl syn::parse::Parse for TraitHListInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let vis = if input.peek(syn::Token![pub]) {
            Some(input.parse()?)
        } else {
            None
        };
        let hlist_trait: syn::Ident = input.parse()?;
        input.parse::<syn::Token![for]>()?;
        input.parse::<syn::Token![trait]>()?;
        let base_trait: syn::Ident = input.parse()?;

        let trait_generic_params = if input.peek(syn::Token![<]) {
            let bracketed: crate::AngleBracketedGenericParams = input.parse()?;
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
                    methods.push(crate::TraitHListMethod::new(trait_item_fn));
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
            vis,
            methods,
            trait_generic_params,
            trait_where_clause,
        })
    }
}

impl TraitHListInput {
    pub fn expand(&self) -> proc_macro2::TokenStream {
        let Self {
            hlist_trait,
            base_trait,
            vis,
            methods,
            trait_generic_params,
            trait_where_clause,
        } = self;

        let method_defs = methods.iter().map(
            |crate::TraitHListMethod {
                 hlist_fn_sig,
                 hlist_output_ident,
                 item_generic_params,
                 item_where_clause,
                 ..
             }| {
                quote::quote! { type #hlist_output_ident <#(#item_generic_params),*> #item_where_clause; #hlist_fn_sig; }
            },
        );
       
        let at_index_method_defs = methods.iter().map(
            |crate::TraitHListMethod {
                 hlist_fn_sig_at_index,
                 ..
             }| { quote::quote! { #hlist_fn_sig_at_index; } },
        );

        let bool_method_defs = methods.iter().filter_map(
            |crate::TraitHListMethod {
                 hlist_fn_sig_all,
                 hlist_fn_sig_any,
                 ..
             }| {
                let (Some(hlist_fn_sig_all), Some(hlist_fn_sig_any)) = (hlist_fn_sig_all, hlist_fn_sig_any) else { return None; };
                Some(quote::quote! {
                    #hlist_fn_sig_all;
                    #hlist_fn_sig_any;
                })
            },
        );

        let allow_unused_variables: syn::Attribute = syn::parse_quote!(#[allow(unused_variables)]);
        let nil_impls = methods.iter().map(
            |crate::TraitHListMethod {
                 hlist_fn_sig,
                 hlist_output_ident,
                 item_generic_params,
                 item_where_clause,
                 ..
             }| {
                quote::quote! {
                    type #hlist_output_ident <#(#item_generic_params),*> = hlist2::Nil #item_where_clause; 
                    #hlist_fn_sig { hlist2::Nil }
                }
            },
        );
        let nil_at_index_impls = methods.iter().map(
            |crate::TraitHListMethod {
                 hlist_fn_sig_at_index,
                 ..
             }| { quote::quote! { #hlist_fn_sig_at_index { panic!("Index out of bounds, expected {__hlist_index} more items in the list.") } } },
        );
        let nil_bool_impls = methods.iter().filter_map(
            |crate::TraitHListMethod {
                 hlist_fn_sig_all,
                 hlist_fn_sig_any,
                 ..
             }| {
                let (Some(hlist_fn_sig_all), Some(hlist_fn_sig_any)) = (hlist_fn_sig_all, hlist_fn_sig_any) else { return None; };
                Some(quote::quote! {
                    #hlist_fn_sig_all {true}
                    #hlist_fn_sig_any {false}
                })
            },
        );

        let cons_impls = methods.iter().map(
            |crate::TraitHListMethod {
                 hlist_fn_sig,
                 item_fn_ident,
                 hlist_fn_ident,
                 item_output,
                 hlist_output_ident,
                 args,
                 args_cloned,
                 item_generic_params,
                item_generic_args,
                 item_where_clause,
                 ..
             }| {
                 quote::quote! {
                    type #hlist_output_ident <#(#item_generic_params),*> 
                        = hlist2::Cons<#item_output, __HListTail::#hlist_output_ident<#(#item_generic_args),*>> #item_where_clause; 
                     #hlist_fn_sig {
                         let hlist2::Cons(__hlist_head, __hlist_tail) = self;
                         hlist2::Cons(__hlist_head.#item_fn_ident(#(#args_cloned),*), __hlist_tail.#hlist_fn_ident(#(#args),*))
                     }
                 }
            },
        );
        let cons_at_index_impls = methods.iter().map(
            |crate::TraitHListMethod {
                 hlist_fn_sig_at_index,
                 item_fn_ident,
                 hlist_fn_ident_at_index,
                 args,
                 ..
             }| {
                quote::quote! {
                    #hlist_fn_sig_at_index {
                        let hlist2::Cons(__hlist_head, __hlist_tail) = self;
                        if __hlist_index == 0 {
                            __hlist_head.#item_fn_ident(#(#args),*)
                        } else {
                            __hlist_tail.#hlist_fn_ident_at_index(#(#args,)* __hlist_index - 1)
                        }
                    }
                }
            },
        );
        let cons_bool_impls = methods.iter().filter_map(
            |crate::TraitHListMethod {
                 hlist_fn_sig_all,
                 hlist_fn_sig_any,
                 hlist_fn_ident_all,
                 hlist_fn_ident_any,
                 args,
                 args_cloned,
                 item_fn_ident,
                 ..
             }| {
                let (Some(hlist_fn_sig_all), Some(hlist_fn_sig_any)) = (hlist_fn_sig_all, hlist_fn_sig_any) else { return None; };
                Some(quote::quote! {
                    #hlist_fn_sig_all {
                        let hlist2::Cons(__hlist_head, __hlist_tail) = self;
                        __hlist_head.#item_fn_ident(#(#args_cloned),*) && __hlist_tail.#hlist_fn_ident_all(#(#args),*)
                    }
                    #hlist_fn_sig_any {
                        let hlist2::Cons(__hlist_head, __hlist_tail) = self;
                        __hlist_head.#item_fn_ident(#(#args_cloned),*) || __hlist_tail.#hlist_fn_ident_any(#(#args),*)
                    }
                })
            },
        );

        let trait_generic_args: Vec<_> = trait_generic_params
            .clone().into_iter()
            .map(crate::generic_param_to_arg::generic_param_to_arg)
            .collect();

        quote::quote! {
            #vis trait #hlist_trait<#(#trait_generic_params),*> #trait_where_clause {
                #(#method_defs)*
                #(#at_index_method_defs)*
                #(#bool_method_defs)*
            }

            #allow_unused_variables
            impl<#(#trait_generic_params),*>
                #hlist_trait<#(#trait_generic_args),*> for hlist2::Nil #trait_where_clause {
                #(#nil_impls)*
                #(#nil_at_index_impls)*
                #(#nil_bool_impls)*
            }
            //
            impl<
                #(#trait_generic_params,)*
                __HListHead: #base_trait <#(#trait_generic_args),*>,
                __HListTail: #hlist_trait<#(#trait_generic_args),*>
            > #hlist_trait<#(#trait_generic_args),*> for hlist2::Cons<__HListHead, __HListTail> #trait_where_clause {
                #(#cons_impls)*
                #(#cons_at_index_impls)*
                #(#cons_bool_impls)*
            }
        }
    }
}
