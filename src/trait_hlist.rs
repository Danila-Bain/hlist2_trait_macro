// #[macro_export]
// macro_rules! impl_trait_hlist {
//     ($TraitHList:ident {
//         trait $Trait {
//
//         }
//     }) => {
//
//     };
// }
//

#[macro_export]
macro_rules! impl_trait_hlist {
    ($hlist_trait:ident {
        trait $base_trait:ident {
            $(fn $fn_name:ident(&self) -> $return_ty:ty;)*
        }
    }) => {
        $crate::paste::paste!(
            trait $hlist_trait {
                $(
                    type [<$fn_name:upper Output>];
                    fn $fn_name(&self) -> Self::[<$fn_name:upper Output>];
                )*

                $(
                    $crate::impl_trait_hlist!(@bool_methods $fn_name $return_ty);
                )*
            }

            impl $hlist_trait for $crate::Nil {
                $(
                    type [<$fn_name:upper Output>] = $crate::Nil;
                    fn $fn_name(&self) -> Self::[<$fn_name:upper Output>] {
                        $crate::Nil
                    }
                )*

                    $(
                        $crate::impl_trait_hlist!(@bool_methods_impl_nil $fn_name $return_ty);
                    )*
            }

            impl<H: $base_trait, T: $hlist_trait> $hlist_trait for $crate::Cons<H, T> {
                $(
                    type [<$fn_name:upper Output>] = $crate::Cons<$return_ty, T::[<$fn_name:upper Output>]>;
                    fn $fn_name(&self) -> Self::[<$fn_name:upper Output>] {
                        Cons(self.head().$fn_name(), self.tail().$fn_name())
                    }
                )*

                    $(
                        $crate::impl_trait_hlist!(@bool_methods_impl_cons $fn_name $return_ty);
                    )*
            }
            );
    };

    (@bool_methods $fn_name:ident bool) => {
        $crate::paste::paste!(
            fn [<all_$fn_name>](&self) -> bool;
            fn [<any_$fn_name>](&self) -> bool;
        );
    };

    (@bool_methods $fn_name:ident $return_ty:ty) => {
        // No extra methods for non-bool return types
    };

    (@bool_methods_impl_nil $fn_name:ident bool) => {
        $crate::paste::paste!(
            fn [<all_$fn_name>](&self) -> bool {
                true
            }
            fn [<any_$fn_name>](&self) -> bool {
                false
            }
        );
    };

    (@bool_methods_impl_nil $fn_name:ident $return_ty:ty) => {
        // No implementation for non-bool return types
    };

    (@bool_methods_impl_cons $fn_name:ident bool) => {
        $crate::paste::paste!(
            fn [<all_$fn_name>](&self) -> bool {
                self.head().$fn_name() && self.tail().[<all_$fn_name>]()
            }
            fn [<any_$fn_name>](&self) -> bool {
                self.head().$fn_name() || self.tail().[<any_$fn_name>]()
            }
        );
    };

    (@bool_methods_impl_cons $fn_name:ident $return_ty:ty) => {
        // No implementation for non-bool return types
    };
}


#[cfg(test)]
mod macro_tests {
    use crate::*;

    #[allow(dead_code)]
    #[test]
    fn empty_traits() {
        trait MyTrait {
            fn a();
        }

        // export nothing
        impl_trait_hlist!(MyTraitHlist {
            trait MyTrait {}
        });
    }

    #[allow(dead_code)]
    #[test]
    fn test() {
        trait MyTrait {
            fn a(&self) -> u32;
            fn b(&self) -> bool;
        }

        impl MyTrait for bool {
            fn a(&self) -> u32 {
                *self as u32
            }
            fn b(&self) -> bool {
                *self
            }
        }

        impl MyTrait for i32 {
            fn a(&self) -> u32 {
                *self as u32
            }
            fn b(&self) -> bool {
                *self != 0
            }
        }

        impl_trait_hlist!(MyTraitHlist {
           trait MyTrait {
               fn a(&self) -> u32;
               fn b(&self) -> bool;
           }
        });

        let hlist = hlist![false, true, 0, 10];
        assert_eq!(hlist![0, 1, 0, 10], hlist.a());
        assert_eq!(hlist![false, true, false, true], hlist.b());
        assert!(!hlist.all_b());
        assert!(hlist.any_b());
        assert!(!hlist![false, false, false].any_b());
        assert!(hlist![true, true, true].all_b());
    }
}

#[cfg(test)]
mod naive_tests {
    use crate::Cons;
    use crate::Nil;

    #[test]
    fn my_trait_no_arguments() {
        trait MyTrait {
            fn a(&self) -> u32;
            fn b(&self) -> bool;
        }

        impl MyTrait for bool {
            fn a(&self) -> u32 {
                *self as u32
            }
            fn b(&self) -> bool {
                *self
            }
        }
        impl MyTrait for i32 {
            fn a(&self) -> u32 {
                *self as u32
            }
            fn b(&self) -> bool {
                *self != 0
            }
        }

        trait MyTraitHList {
            type AOutput;
            fn a(&self) -> Self::AOutput;
            type BOutput;
            fn b(&self) -> Self::BOutput;

            fn all_b(&self) -> bool;
            fn any_b(&self) -> bool;
        }
        impl MyTraitHList for Nil {
            type AOutput = Nil;
            fn a(&self) -> Self::AOutput {
                Nil
            }
            type BOutput = Nil;
            fn b(&self) -> Self::BOutput {
                Nil
            }

            fn all_b(&self) -> bool {
                true
            }
            fn any_b(&self) -> bool {
                false
            }
        }
        impl<H: MyTrait, T: MyTraitHList> MyTraitHList for Cons<H, T> {
            type AOutput = Cons<u32, T::AOutput>;
            fn a(&self) -> Self::AOutput {
                Cons(self.head().a(), self.tail().a())
            }
            type BOutput = Cons<bool, T::BOutput>;
            fn b(&self) -> Self::BOutput {
                Cons(self.head().b(), self.tail().b())
            }

            fn all_b(&self) -> bool {
                self.head().b() && self.tail().all_b()
            }
            fn any_b(&self) -> bool {
                self.head().b() || self.tail().any_b()
            }
        }
        let hlist = Cons(false, Cons(true, Cons(0, Cons(10, Nil))));
        assert_eq!(Cons(0, Cons(1, Cons(0, Cons(10, Nil)))), hlist.a());
        assert_eq!(
            Cons(false, Cons(true, Cons(false, Cons(true, Nil)))),
            hlist.b()
        );
        assert!(!hlist.all_b());
        assert!(hlist.any_b());
        assert!(!Cons(false, Cons(false, Nil)).any_b());
        assert!(Cons(true, Cons(true, Nil)).all_b());
    }

    #[test]
    fn my_trait_copy_argument() {
        trait MyTrait {
            fn a(&self, x: f64) -> u32;
            fn b(&self, x: f64) -> bool;
        }

        impl MyTrait for bool {
            fn a(&self, x: f64) -> u32 {
                *self as u32 + x as u32
            }
            fn b(&self, x: f64) -> bool {
                *self || x != 0.
            }
        }
        impl MyTrait for i32 {
            fn a(&self, x: f64) -> u32 {
                *self as u32 + x as u32
            }
            fn b(&self, x: f64) -> bool {
                *self != 0 || x != 0.
            }
        }

        trait MyTraitHList {
            type AOutput;
            fn a(&self, x: f64) -> Self::AOutput;
            type BOutput;
            fn b(&self, x: f64) -> Self::BOutput;

            fn all_b(&self, x: f64) -> bool;
            fn any_b(&self, x: f64) -> bool;
        }
        impl MyTraitHList for Nil {
            type AOutput = Nil;
            fn a(&self, _x: f64) -> Self::AOutput {
                Nil
            }
            type BOutput = Nil;
            fn b(&self, _x: f64) -> Self::BOutput {
                Nil
            }

            fn all_b(&self, _x: f64) -> bool {
                true
            }
            fn any_b(&self, _x: f64) -> bool {
                false
            }
        }
        impl<H: MyTrait, T: MyTraitHList> MyTraitHList for Cons<H, T> {
            type AOutput = Cons<u32, T::AOutput>;
            fn a(&self, x: f64) -> Self::AOutput {
                Cons(self.head().a(x.clone()), self.tail().a(x))
            }
            type BOutput = Cons<bool, T::BOutput>;
            fn b(&self, x: f64) -> Self::BOutput {
                Cons(self.head().b(x.clone()), self.tail().b(x))
            }

            fn all_b(&self, x: f64) -> bool {
                self.head().b(x.clone()) && self.tail().all_b(x)
            }
            fn any_b(&self, x: f64) -> bool {
                self.head().b(x.clone()) || self.tail().any_b(x)
            }
        }
        let hlist = Cons(false, Cons(true, Cons(0, Cons(10, Nil))));
        assert_eq!(Cons(0, Cons(1, Cons(0, Cons(10, Nil)))), hlist.a(0.));
        assert_eq!(
            Cons(false, Cons(true, Cons(false, Cons(true, Nil)))),
            hlist.b(0.)
        );
        assert!(!hlist.all_b(0.));
        assert!(hlist.all_b(1.));
        assert!(hlist.any_b(0.));
        assert!(!Cons(false, Cons(false, Nil)).any_b(0.));
        assert!(Cons(true, Cons(true, Nil)).all_b(0.));
    }

    #[test]
    fn my_trait_generic_copy_argument() {
        trait MyTrait<X: Into<u32>> {
            fn a(&self, x: X) -> u32;
            fn b(&self, x: X) -> bool;
        }

        impl<X: Into<u32>> MyTrait<X> for bool {
            fn a(&self, x: X) -> u32 {
                *self as u32 + x.into()
            }
            fn b(&self, x: X) -> bool {
                *self || x.into() != 0
            }
        }
        impl<X: Into<u32>> MyTrait<X> for i32 {
            fn a(&self, x: X) -> u32 {
                *self as u32 + x.into()
            }
            fn b(&self, x: X) -> bool {
                *self != 0 || x.into() != 0
            }
        }

        trait MyTraitHList<X: Into<u32>> {
            type AOutput;
            fn a(&self, x: X) -> Self::AOutput;
            type BOutput;
            fn b(&self, x: X) -> Self::BOutput;

            fn all_b(&self, x: X) -> bool;
            fn any_b(&self, x: X) -> bool;
        }
        impl<X: Into<u32>> MyTraitHList<X> for Nil {
            type AOutput = Nil;
            fn a(&self, _x: X) -> Self::AOutput {
                Nil
            }
            type BOutput = Nil;
            fn b(&self, _x: X) -> Self::BOutput {
                Nil
            }

            fn all_b(&self, _x: X) -> bool {
                true
            }
            fn any_b(&self, _x: X) -> bool {
                false
            }
        }
        impl<X: Into<u32> + Clone, H: MyTrait<X>, T: MyTraitHList<X>> MyTraitHList<X> for Cons<H, T> {
            type AOutput = Cons<u32, T::AOutput>;
            fn a(&self, x: X) -> Self::AOutput {
                Cons(self.head().a(x.clone()), self.tail().a(x))
            }
            type BOutput = Cons<bool, T::BOutput>;
            fn b(&self, x: X) -> Self::BOutput {
                Cons(self.head().b(x.clone()), self.tail().b(x))
            }

            fn all_b(&self, x: X) -> bool {
                self.head().b(x.clone()) && self.tail().all_b(x)
            }
            fn any_b(&self, x: X) -> bool {
                self.head().b(x.clone()) || self.tail().any_b(x)
            }
        }
        let hlist = Cons(false, Cons(true, Cons(0, Cons(10, Nil))));
        assert_eq!(Cons(0, Cons(1, Cons(0, Cons(10, Nil)))), hlist.a(0u16));
        assert_eq!(
            Cons(false, Cons(true, Cons(false, Cons(true, Nil)))),
            hlist.b(0u8)
        );
        assert!(!hlist.all_b(0u8));
        assert!(hlist.all_b(true));
        assert!(hlist.any_b(0u16));
        assert!(!Cons(false, Cons(false, Nil)).any_b(false));
        assert!(Cons(true, Cons(true, Nil)).all_b(0u32));
    }

    #[test]
    fn my_trait_no_argument_dyn_implementation() {
        trait MyTrait {
            fn a(&self) -> u32;
            fn b(&self) -> bool;
        }

        impl MyTrait for bool {
            fn a(&self) -> u32 {
                *self as u32
            }
            fn b(&self) -> bool {
                *self
            }
        }
        impl MyTrait for i32 {
            fn a(&self) -> u32 {
                *self as u32
            }
            fn b(&self) -> bool {
                *self != 0
            }
        }

        trait MyTraitHList {
            type AOutput;
            fn a(&self) -> Self::AOutput;
            type BOutput;
            fn b(&self) -> Self::BOutput;

            fn all_b(&self) -> bool;
            fn any_b(&self) -> bool;
        }
        impl MyTraitHList for Nil {
            type AOutput = Nil;
            fn a(&self) -> Self::AOutput {
                Nil
            }
            type BOutput = Nil;
            fn b(&self) -> Self::BOutput {
                Nil
            }

            fn all_b(&self) -> bool {
                true
            }
            fn any_b(&self) -> bool {
                false
            }
        }
        impl<H: MyTrait, T: MyTraitHList> MyTraitHList for Cons<H, T> {
            type AOutput = Cons<u32, T::AOutput>;
            fn a(&self) -> Self::AOutput {
                Cons(self.head().a(), self.tail().a())
            }
            type BOutput = Cons<bool, T::BOutput>;
            fn b(&self) -> Self::BOutput {
                Cons(self.head().b(), self.tail().b())
            }

            fn all_b(&self) -> bool {
                self.head().b() && self.tail().all_b()
            }
            fn any_b(&self) -> bool {
                self.head().b() || self.tail().any_b()
            }
        }

        impl<const N: usize, MyTraitAsRef: AsRef<dyn MyTrait>> MyTraitHList for [MyTraitAsRef; N] {
            type AOutput = [u32; N];
            fn a(&self) -> Self::AOutput {
                std::array::from_fn(|i| self[i].as_ref().a())
            }
            type BOutput = [bool; N];
            fn b(&self) -> Self::BOutput {
                std::array::from_fn(|i| self[i].as_ref().b())
            }

            fn all_b(&self) -> bool {
                for i in 0..N {
                    if !self[i].as_ref().b() {
                        return false;
                    }
                }
                return true;
            }
            fn any_b(&self) -> bool {
                for i in 0..N {
                    if self[i].as_ref().b() {
                        return true;
                    }
                }
                return false;
            }
        }

        impl<MyTraitAsRef: AsRef<dyn MyTrait>> MyTraitHList for Vec<MyTraitAsRef> {
            type AOutput = Vec<u32>;
            fn a(&self) -> Self::AOutput {
                self.into_iter().map(|i| i.as_ref().a()).collect()
            }
            type BOutput = Vec<bool>;
            fn b(&self) -> Self::BOutput {
                self.into_iter().map(|i| i.as_ref().b()).collect()
            }

            fn all_b(&self) -> bool {
                self.into_iter().all(|i| i.as_ref().b())
            }
            fn any_b(&self) -> bool {
                self.into_iter().any(|i| i.as_ref().b())
            }
        }

        let hlist = Cons(false, Cons(true, Cons(0, Cons(10, Nil))));
        assert_eq!(Cons(0, Cons(1, Cons(0, Cons(10, Nil)))), hlist.a());
        assert_eq!(
            Cons(false, Cons(true, Cons(false, Cons(true, Nil)))),
            hlist.b()
        );
        assert!(!hlist.all_b());
        assert!(hlist.any_b());
        assert!(!Cons(false, Cons(false, Nil)).any_b());
        assert!(Cons(true, Cons(true, Nil)).all_b());

        let array: [Box<dyn MyTrait>; _] =
            [Box::new(false), Box::new(true), Box::new(0), Box::new(10)];
        assert_eq!([0, 1, 0, 10], array.a());
        assert_eq!([false, true, false, true], array.b());
        assert!(!array.all_b());
        assert!(array.any_b());

        let vec: Vec<Box<dyn MyTrait>> =
            vec![Box::new(false), Box::new(true), Box::new(0), Box::new(10)];
        assert_eq!(vec![0, 1, 0, 10], vec.a());
        assert_eq!(vec![false, true, false, true], vec.b());
        assert!(!vec.all_b());
        assert!(vec.any_b());
    }
}
