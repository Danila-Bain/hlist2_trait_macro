mod angle_bracketed_generic_params;
use angle_bracketed_generic_params::AngleBracketedGenericParams;

/// Macro, that generates trait implementations for heterogeneous lists
/// whose elements share provided trait.
///
/// The `TraitHList!` macro automatically generates trait implementations 
/// for heterogeneous lists (`hlist!` from the `hlist2` crate), allowing 
/// trait methods to be applied elementwise across all list elements. 
///
/// It supports traits with arbitrary generics, lifetimes, const parameters,
/// and `where` clauses, as well as methods with any receiver form 
/// (`self`, `&self`, `&mut self`) and arbitrary parameter types.
///
/// The macro defines a new trait (e.g. `MyTraitHlist`) mirroring 
/// the methods of the original one (e.g. `MyTrait`). Implemented
/// of types that implement the source trait. Each listed method produces
/// an `hlist!` of results, preserving element order. 
/// Methods returning `bool` automatically gain two aggregators: 
/// `.all_<method>()` and `.any_<method>()`. 
///
/// Individual methods can be renamed with `#[name = ...]`.
///
/// In essence, `TraitHList!` extends any trait to operate 
/// seamlessly over heterogeneous lists, as a replacement for lacking 
/// iteration capabilities.
///
///
/// ## Basic Usage
///
/// ```rust
/// TraitHList!{
///     HListTraitName for trait TraitName { 
///         <methods of TraitName without default implementations>
///     }
/// };
/// ```
///
/// ```rust
/// use hlist2::hlist;
/// use hlist2_trait_macro::TraitHList;
///
/// trait MyTrait {
///     fn to_u32(&self) -> u32;
///     fn to_bool(&self) -> bool;
/// }
///
/// impl MyTrait for bool {
///     fn to_u32(&self) -> u32 { *self as u32 }
///     fn to_bool(&self) -> bool { *self }
/// }
///
/// impl MyTrait for i32 {
///     fn to_u32(&self) -> u32 { *self }
///     fn to_bool(&self) -> bool { *self != 0 }
/// }
///
/// TraitHList!(
///     MyTraitHList for trait MyTrait {
///         fn to_u32(&self) -> u32;
///         fn to_bool(&self) -> bool;
///     }
/// );
///
/// let l = hlist![false, true, 0, 10];
/// assert_eq!(hlist![0, 1, 0, 10], l.to_u32());
/// assert_eq!(hlist![false, true, false, true], l.to_bool());
/// assert!(!l.all_to_bool());
/// assert!(l.any_to_bool());
/// ```
///
/// - The macro defines a trait `MyTraitHList` and implements it 
///   for all `hlist!` combinations of types that implement `MyTrait`.
/// - Each method in the `MyTraitHList` acts **elementwise** on the list:
///   - `l.to_u32()` calls `to_u32()` on each element.
///   - `l.to_bool()` does the same.
/// - For methods that return bool, macro also provides:
///   - `.all_<method>()` — returns `true` if all results are `true`.
///   - `.any_<method>()` — returns `true` if any result is `true`.
///   `.all_` and `.any_` methods are lazily evaluated from head to tail.
///
/// ## Renaming Methods
///
/// Each method can be renamed in the HList version 
/// using attribute `#[name = ...]`, which can be
/// usefull to avoid naming collisions.
/// ```rust
/// TraitHList! {
///     IntoHlist for trait Into<T> {
///         #[name = hlist_into]
///         fn into(self) -> T;
///     }
/// }
///
/// let list = hlist![true, 1u8, 1u16, 1u32];
/// assert_eq!(hlist![1u64, 1u64, 1u64, 1u64], list.hlist_into());
/// ```
///
///
/// This generates a method `hlist_into` instead of the default `into`.
///
/// ## Generic Traits
///
/// ```rust
/// trait MyTrait<const N: usize, T: Into<i64>> {
///     fn a<U: Into<i64>>(&self, x: i64, y: U, z: T) -> bool;
///     fn b(self, x: i64, y: &i64, z: T) -> bool;
/// }
///
/// impl<const N: usize, T: Into<i64>> MyTrait<N, T> for [i64; N] {
///     fn a<U: Into<i64>>(&self, x: i64, y: U, z: T) -> bool {
///         (self.into_iter().sum::<i64>() + x + y.into() + z.into()) == 0
///     }
///     fn b(self, x: i64, y: &i64, z: T) -> bool {
///         (self.into_iter().sum::<i64>() + x + y + z.into()) == 0
///     }
/// }
///
/// TraitHList! {
///     MyTraitHlist for trait MyTrait<const N: usize, T: Into<i64>> {
///         fn a<U: Into<i64>>(&self, x: i64, y: U, z: T) -> bool where T: Copy, U: Copy;
///         fn b(self, x: i64, y: &i64, z: T) -> bool where T: Clone;
///     }
/// }
///
/// // Note that size must be the same, because N is the parameter of the trait, not methods
/// let h0 = hlist![[0; 4], [1; 4], [2; 4], [3; 4], [4; 4],];
///
/// assert_eq!(
///     hlist![false, true, false, false, false],
///     h0.a(0i64, 4u32, -8i16)
/// );
/// assert_eq!(
///     hlist![false, true, false, false, false],
///     h0.b(0i64, &4i64, -8i16)
/// );
/// ```
///
/// Generated methods will operate on `hlist!`s of arrays `[i64; N]` with consistent `N`.
///
///
/// Also note, that paramters passed by value must implement either `Copy` or `Clone`, 
/// because they are passed to each element of the list.
///
/// ## Comments and Unused Methods
///
/// Any methods omitted in the macro definition are ignored.  
/// Comments are safely ignored as well.
///
/// ---
///
/// ## Summary of Features
///
/// | Feature                            | Supported | Description |
/// |------------------------------------|------------|--------------|
/// | Elementwise trait method calls     | ✅ | Applies trait methods to each list element |
/// | Arbitrary trait-level generics and bounds | ✅ | Generic, const, lifetime parameters |
/// | Trait-level `where` clauses        | ✅ | Fully supported |
/// | Arbitrary method-level generics and bounds | ⚠️ | Everywhere exept in return type |
/// | Method-level `where` clauses             | ✅ | Fully supported  |
/// | Different receiver forms           | ✅ | `self`, `&self`, `&mut self` |
/// | Method renaming                    | ✅ | `#[name = ...]` attribute |
/// | Additional convenience methods     | ✅ | `any_*`, `all_*` for `bool`-returning methods |
/// | Comments in macro body             | ✅ | Ignored |
/// | Associated types in traits | ⛔ | Not planned until usecase if found |
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
