pub trait Indexed: Copy {
    const COUNT: usize;
    fn index(self) -> usize;
    fn from_index(index: usize) -> Self;
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
