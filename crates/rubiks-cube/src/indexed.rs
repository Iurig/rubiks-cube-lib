pub trait Indexed: Copy {
    const COUNT: usize;
    fn index(self) -> usize;
    fn from_index(index: usize) -> Self;

    fn all() -> impl Iterator<Item = Self> + Clone {
        (0..Self::COUNT).map(Self::from_index)
    }
}

impl<A: Indexed, B: Indexed> Indexed for (A, B) {
    const COUNT: usize = A::COUNT * B::COUNT;
    fn index(self) -> usize {
        self.0.index() * B::COUNT + self.1.index()
    }
    fn from_index(index: usize) -> Self {
        (
            A::from_index(index / B::COUNT),
            B::from_index(index % B::COUNT),
        )
    }
}

/// Checks the round-trip law for every index of `T`: `from_index` and `index` undo each other.
#[cfg(test)]
pub fn assert_round_trips<T: Indexed + Eq + std::fmt::Debug>() {
    for i in 0..T::COUNT {
        let value = T::from_index(i);
        assert_eq!(value.index(), i, "{value:?}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct Three(usize);
    impl Indexed for Three {
        const COUNT: usize = 3;
        fn index(self) -> usize {
            self.0
        }
        fn from_index(index: usize) -> Self {
            assert!(index < Self::COUNT, "{index} is not below {}", Self::COUNT);
            Self(index)
        }
    }

    #[test]
    fn a_pair_numbers_every_combination_once() {
        assert_eq!(<(Three, Three)>::COUNT, 9);
        assert_round_trips::<(Three, Three)>();
    }
}
