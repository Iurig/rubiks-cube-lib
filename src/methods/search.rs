use std::collections::HashMap;

use crate::{Puzzle, methods::simple_methods::SimpleMask};

pub struct BFSMemo<P: Puzzle> {
    memorization: HashMap<SimpleMask<P>, Vec<P::Moves>>,
    to_deepen: Vec<SimpleMask<P>>,
    depth: usize,
}

impl<P: Puzzle> BFSMemo<P> {
    pub fn new_empty() -> Self {
        Self {
            memorization: HashMap::new(),
            to_deepen: vec![],
            depth: 0,
        }
    }

    pub fn initialize(mask: SimpleMask<P>) -> Self {
        Self {
            memorization: HashMap::from([(mask.clone(), Vec::new())]),
            to_deepen: vec![mask],
            depth: 0,
        }
    }

    fn search_to(depth: usize) -> Self {
        todo!()
    }
}
