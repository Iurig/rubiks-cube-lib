/*use std::{collections::HashMap, marker::PhantomData};

use crate::{NamedMoveSequences, Puzzle, SimpleStep, SolveMethod, SolveStep};

type Mask<P> = Vec<(<P as Puzzle>::Pieces, usize)>;

pub struct BFSMemo<P: Puzzle, M: SolveMethod<P, NamedMoveSequences<P>>> {
    memorization: HashMap<Mask<P>, Vec<P::Moves>>,
    to_deepen: Vec<Mask<P>>,
    depth: usize,
    phantom: PhantomData<M>,
}

impl<P: Puzzle, M: SolveMethod<P, NamedMoveSequences<P>>> BFSMemo<P, M> {
    pub fn new(step: SimpleStep<P, M>) -> Self {
        BFSMemo::<P, M> {
            memorization: HashMap::from([(step.mask(&P::default()), Vec::new())]),
            to_deepen: vec![step.mask(&P::default())],
            depth: 0,
            phantom: PhantomData,
        }
    }
    fn search_to(depth: usize) -> Self {
        todo!()
    }
}
*/
