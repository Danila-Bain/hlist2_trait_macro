#[macro_export]
macro_rules! hlist {
    () => { $crate::Nil };
    ($first:expr $(, $rest:expr)* $(,)?) => {
        $crate::Cons($first, $crate::hlist!($($rest),*))
    };
}
