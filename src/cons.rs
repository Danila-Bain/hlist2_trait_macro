/// Heterogenous list with head and tail values, where tail is another heterogenous list.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Default)]
pub struct Cons<Head, Tail>(pub Head, pub Tail);


impl<Head, Tail> Cons<Head, Tail> {
    /// Constructs a new [`Cons`] with provided head and tail values.
    pub const fn new(head: Head, tail: Tail) -> Self {
        Self(head, tail)
    }

    pub fn into_head(self) -> Head {
        let Self(head, _) = self;
        head
    }
    pub fn into_tail(self) -> Tail {
        let Self(_, tail) = self;
        tail
    }
    pub fn head(&self) -> &Head {
        let Self(head, _) = self;
        head
    }
    pub fn tail(&self) -> &Tail {
        let Self(_, tail) = self;
        tail
    }
    pub fn head_mut(&mut self) -> &mut Head {
        let Self(head, _) = self;
        head
    }
    pub fn tail_mut(&mut self) -> &mut Tail {
        let Self(_, tail) = self;
        tail
    }
}

impl<Head, Tail> From<(Head, Tail)> for Cons<Head, Tail> {
    fn from(value: (Head, Tail)) -> Self {
        let (head, tail) = value;
        Self(head, tail)
    }
}

impl<Head, Tail> From<Cons<Head, Tail>> for (Head, Tail) {
    fn from(value: Cons<Head, Tail>) -> Self {
        let Cons(head, tail) = value;
        (head, tail)
    }
}
