/// Trait for types with few variants that are all accessible from a comprehensive array. So they
/// can be iterated from, and a permutation can be expressed simply as another array.
pub trait Indexed: Copy {
    /// How many values the type can assume.
    const COUNT: usize;
    /// Where the value is indexed in the comprehensive array.
    fn index(self) -> usize;
    /// What value lives in a given index of the array.
    ///
    /// # Panics
    /// Panics if the index is not in the range `0..COUNT`.
    fn from_index(index: usize) -> Self;

    /// Iterator over all values of the type.
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
