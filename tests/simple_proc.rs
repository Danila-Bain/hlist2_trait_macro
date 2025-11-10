use trait_hlist::*;

#[test]
fn simple_proc() {
    #[allow(dead_code)]
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

    impl_trait_hlist_proc!(
        MyTraitHlist for trait MyTrait {
            fn a(&self) -> u32;
            fn b(&self) -> bool;
        }
    );

    // let hlist = hlist![false, true, 0, 10];
    // assert_eq!(hlist![0, 1, 0, 10], hlist.a());
    // assert_eq!(hlist![false, true, false, true], hlist.b());
    // assert!(!hlist.all_b());
    // assert!(hlist.any_b());
    // assert!(!hlist![false, false, false].any_b());
    // assert!(hlist![true, true, true].all_b());
}

#[test]
fn parameters_in_methods() {
    #[allow(dead_code)]
    trait MyTrait {
        fn a(&self, x: &f64, y: &mut u32, z: u8) -> u32;
        fn b(&mut self, x: &f64, y: &mut u32, z: u8) -> bool;
        fn c(self, x: &f64, y: &mut u32, z: u8) -> bool;
        fn extra_one_ignored(self) -> Self;
    }

    impl_trait_hlist_proc!(
        MyTraitHlist for trait MyTrait {
            fn a(&self, x: &f64, y: &mut u32, z: u8) -> u32;
            fn b(&mut self, x: &f64, y: &mut u32, z: u8) -> bool;
            fn c(self, x: &f64, y: &mut u32, z: u8) -> bool;
        }
    );
}

#[test]
fn generic_parameters_in_methods() {
    #[allow(dead_code)]
    trait MyTrait {
        fn a(&self, x: impl Copy) -> u32;
        fn b<T>(&self, x: T) -> bool;
        fn c<T>(&self, x: T) -> bool
            where T: std::fmt::Display;
        fn extra_one_ignored(self) -> Self;
    }

    impl_trait_hlist_proc!(
        MyTraitHlist for trait MyTrait {
            fn a(&self, x: impl Copy) -> u32;
            fn b<T: Copy>(&self, x: T) -> bool;
            fn c<T>(&self, x: T) -> bool
                where T: std::fmt::Display + Copy;
        }
    );
}


#[test]
fn generic_trait() {
    #[allow(dead_code)]
    trait MyTrait<'a, T> {
        fn a(&self, x: T) -> bool;
        fn b(&self, x: T) -> bool;
        fn extra_one_ignored(self) -> Self;
    }

    impl_trait_hlist_proc!(
        MyTraitHlist for trait MyTrait<'a, T: std::fmt::Display> where T: Copy {
            fn a(&self, x: T) -> bool;
            fn b(&self, x: T) -> bool;
        }
    );
}

#[test]
fn generic_trait_2() {
    #[allow(dead_code)]
    trait MyTrait<'a, const N: usize, T> {
        fn a<'aa: 'a>(&'a self, x: &'aa T, y: [T; N]) -> bool;
        fn b(&self, x: T, y: [T; N]) -> bool;
        fn extra_one_ignored(self) -> Self;
    }

    impl_trait_hlist_proc!(
        MyTraitHlist for trait MyTrait<'a, const N: usize, T: std::fmt::Display> where {
            fn a<'aa: 'a>(&'a self, x: &'aa T, y: [T; N]) -> bool where T: Copy;
            fn b(&self, x: T, y: [T; N]) -> bool where T: Copy;
        }
    );
}

