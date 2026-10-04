use std::{collections::HashSet, fmt::Debug, hash::Hash, iter::IntoIterator, ops::Mul};

use crate::{AlgSet, Puzzle};

/// A puzzle with optional labels attatched to its pieces, as well as optional orientations.
///
/// Generalizes the concept of a partial puzzle - in which the labels are the name of the pieces
/// themselves, and a puzzle that a set of pieces is tracked without regard for their identities -
/// where the labels are anything of unit type, and everything inbetween.
///
/// For partial puzzles, use [`Mask`](crate::Mask), for pieces tracked with a generic mark, use
/// [`Marked`](crate::Marked)
#[derive(Clone, Eq, Debug, Hash, PartialEq)]
pub struct Labeled<P: Puzzle, L: Marker<P>>(Box<[SlotCondition<P, L>]>);

/// A partly defined puzzle state, usually read as a condition: which pieces must sit in their
/// home slots, and which slots must hold an oriented piece.
///
/// [`SearchStep`](crate::SearchStep)s use masks for where they start and what they solve. A mask
/// ignores every piece it does not name.
///
/// ```
/// use rubiks_cube::{Cube3x3, Edge, Mask, Pieces3x3};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let uf_solved = Mask::<Cube3x3>::new_from_pieces([Pieces3x3::Edge(Edge::Uf)]);
/// assert!(uf_solved.applies_to(&Cube3x3::from_solved("R")?));
/// assert!(!uf_solved.applies_to(&Cube3x3::from_solved("U")?));
/// # Ok(())
/// # }
/// ```
pub type Mask<P> = Labeled<P, ByPiece>;

/// A puzzle state with optional orientations, and optional tracking of specific pieces.
///
/// Used for defining steps by marking positions to be solved, as well as to track cubes throughout
/// moves without distinction for it's marked pieces.
pub type Marked<P> = Labeled<P, ByMark>;

/// What a [`Mask`] asks of one slot: which piece must sit there, and which orientation the piece
/// there must have. `None` asks nothing.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SlotCondition<P: Puzzle, L: Marker<P>> {
    pub(crate) label: Option<L::Label>,
    pub(crate) orient: Option<P::Orientation>,
}

/// What a [`Labeled`] writes in a slot. Implemented by marker types instead of by the label types
/// themselves, so the impls for [`ByPiece`] and [`Tracked`] cannot overlap even if some puzzle's
/// piece type is `()`.
pub trait Marker<P: Puzzle> {
    /// The label stored in each slot.
    type Label: Copy + Eq + Hash + Debug;
    /// The label meaning "this slot's own piece belongs here".
    fn home(slot: P::Piece) -> Self::Label;
}

/// Labels are pieces: each slot names the piece that must sit there. [`Mask`] uses it.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ByPiece;

/// Labels are `()`: a labeled slot's own piece must end up home. Goals use it.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ByMark;

impl<P: Puzzle> Marker<P> for ByPiece {
    type Label = P::Piece;
    fn home(slot: P::Piece) -> Self::Label {
        slot
    }
}

impl<P: Puzzle> Marker<P> for ByMark {
    type Label = ();
    fn home(_slot: P::Piece) -> Self::Label {}
}

impl<P: Puzzle, L: Marker<P>> Default for Labeled<P, L> {
    fn default() -> Self {
        Self::new_empty()
    }
}

impl<P: Puzzle, L: Marker<P>> Mul<P::Moves> for Labeled<P, L> {
    type Output = Self;
    fn mul(self, rhs: P::Moves) -> Self::Output {
        self.composed_with(&(P::default() * rhs))
    }
}

impl<P: Puzzle, L: Marker<P>> Labeled<P, L> {
    pub(crate) fn from_fn(mut condition: impl FnMut(P::Piece) -> SlotCondition<P, L>) -> Self {
        Self(P::ALL_PIECES.iter().map(|&slot| condition(slot)).collect())
    }

    /// A mask that names no piece, so every puzzle meets it. The same as `Mask::default()`.
    #[must_use]
    pub fn new_empty() -> Self {
        Self::from_fn(|_| SlotCondition {
            label: None,
            orient: None,
        })
    }

    /// A mask where each piece in `pieces` must be solved: home and oriented. Same as
    /// <code>[from_double_iter](Self::from_double_iter)(pieces.clone(), pieces)</code>.
    pub fn new_from_pieces<I>(pieces: I) -> Self
    where
        I: IntoIterator<Item = P::Piece> + Clone,
    {
        Self::from_double_iter(pieces.clone(), pieces)
    }

    /// A mask where each piece in `permutations` must sit in its home slot, and the home slot of
    /// each piece in `orientations` must hold an oriented piece.
    ///
    /// The two lists are independent. A piece only in `orientations` asks that whatever piece
    /// sits in its slot be oriented, which is how Roux's corner orientation step (`CO`) says
    /// "orient the last-layer corners, in any order".
    pub fn from_double_iter<I1, I2>(permutations: I1, orientations: I2) -> Self
    where
        I1: IntoIterator<Item = P::Piece>,
        I2: IntoIterator<Item = P::Piece>,
    {
        let perm_iter: HashSet<<P as Puzzle>::Piece> =
            HashSet::<<P as Puzzle>::Piece>::from_iter(permutations);
        let orient_iter: HashSet<<P as Puzzle>::Piece> =
            HashSet::<<P as Puzzle>::Piece>::from_iter(orientations);
        Self::from_fn(|slot| SlotCondition {
            label: perm_iter.contains(&slot).then_some(L::home(slot)),
            orient: orient_iter
                .contains(&slot)
                .then_some(P::default().orientation_at(&slot)),
        })
    }

    #[must_use]
    pub(crate) fn filter_by_piece(puzzle: &P, goal: &Marked<P>) -> Self {
        Self::from_fn(|slot| {
            let label = L::home(puzzle.piece_at(&slot));
            let tracked = goal.condition(puzzle.piece_at(&slot)).label.is_some();
            SlotCondition {
                label: tracked.then_some(label),
                // A goal orientation belongs to the fixed slot only when the goal names no
                // piece there; otherwise it belongs to that piece and moves with it.
                orient: (tracked
                    || (goal.condition(slot).orient.is_some()
                        && goal.condition(slot).label.is_none()))
                .then(|| puzzle.orientation_at(&slot)),
            }
        })
    }

    /// What this mask asks of `slot`.
    #[expect(
        clippy::indexing_slicing,
        reason = "every constructor gives a mask one condition per entry of `P::ALL_PIECES`, and `P::index` returns a position in it"
    )]
    pub(crate) fn condition(&self, slot: P::Piece) -> &SlotCondition<P, L> {
        &self.0[P::index(slot)]
    }

    /// This mask carried along by `puzzle`: each slot takes the condition of the slot its piece
    /// came from, so labels and orientation requirements travel with the pieces. Applying moves
    /// one at a time with `*` gives the same result as composing once with their product.
    pub(crate) fn composed_with(&self, puzzle: &P) -> Self {
        Self::from_fn(|slot| {
            let source = self.condition(puzzle.piece_at(&slot));
            SlotCondition {
                label: source.label,
                orient: source.orient.map(|o| o + puzzle.orientation_at(&slot)),
            }
        })
    }

    /// A mask where each piece in `permutations` must sit in its home slot, and the home slot of
    /// each piece in `orientations` must hold an oriented piece.
    ///
    /// The two lists are independent. A piece only in `orientations` asks that whatever piece
    /// sits in its slot be oriented, which is how Roux's corner orientation step (`CO`) says
    /// "orient the last-layer corners, in any order".
    pub fn from_iter<I1, I2>(labels: I1, orientations: I2) -> Self
    where
        I1: IntoIterator<Item = (P::Piece, L::Label)>,
        I2: IntoIterator<Item = P::Piece>,
    {
        let perm_iter = HashSet::<(<P as Puzzle>::Piece, L::Label)>::from_iter(labels);
        let orient_iter: HashSet<<P as Puzzle>::Piece> =
            HashSet::<<P as Puzzle>::Piece>::from_iter(orientations);
        Self::from_fn(|slot| SlotCondition {
            label: perm_iter
                .iter()
                .find_map(|(curr_slot, label)| (*curr_slot == slot).then_some(*label)),
            orient: orient_iter
                .contains(&slot)
                .then_some(P::default().orientation_at(&slot)),
        })
    }
}

impl<P: Puzzle> Mask<P> {
    /// Whether `puzzle` meets every condition in this mask.
    #[must_use]
    pub fn applies_to(&self, puzzle: &P) -> bool {
        self.0.iter().zip(P::ALL_PIECES).all(|(condition, slot)| {
            condition
                .label
                .is_none_or(|piece| puzzle.piece_at(slot) == piece)
                && condition
                    .orient
                    .is_none_or(|orient| puzzle.orientation_at(slot) == orient)
        })
    }
}

impl<P: Puzzle> Marked<P> {
    /// Composes two instances of [`Marked<P>`], marking pieces if they are tracked by either one.
    ///
    /// Zeroes the orientations that are tracked - is meant to be used in goal-type values.
    #[must_use]
    pub fn or(&self, other: &Self) -> Self {
        Self::from_double_iter(
            P::ALL_PIECES
                .iter()
                .zip(self.0.iter().zip(other.0.clone()))
                .filter(|&(_, (con_self, ref con_other))| {
                    con_self.label.is_some() || con_other.label.is_some()
                })
                .map(|(&slot, _)| slot),
            P::ALL_PIECES
                .iter()
                .zip(self.0.iter().zip(other.0.clone()))
                .filter(|&(_, (con_self, ref con_other))| {
                    con_self.orient.is_some() || con_other.orient.is_some()
                })
                .map(|(&slot, _)| slot),
        )
    }

    /// Whether `puzzle` has the slots marked by tracked solved.
    #[must_use]
    pub fn applies_to(&self, puzzle: &P) -> bool {
        self.0.iter().zip(P::ALL_PIECES).all(|(condition, slot)| {
            condition
                .label
                .is_none_or(|()| puzzle.piece_at(slot) == *slot)
                && condition
                    .orient
                    .is_none_or(|orient| puzzle.orientation_at(slot) == orient)
        })
    }

    /// Constructs a [`Marked Puzzle`](Marked<P>) by checking which pieces cannot be moved using
    /// only an algset.
    #[expect(
        clippy::indexing_slicing,
        reason = "Out of bounds slice is an unrecoverable state"
    )]
    pub fn from_algset(algset: &AlgSet<P>) -> Self {
        let mut mask = Mask::<P>::new_from_pieces(P::ALL_PIECES.to_vec());
        let mut changed = true;
        while changed {
            changed = false;
            for alg in algset.algs() {
                let moved = alg.iter().fold(mask.clone(), |p, m| p * *m);
                for &p in P::ALL_PIECES {
                    if mask.condition(p).label != moved.condition(p).label {
                        mask.0[P::index(p)].label = None;
                        changed = true;
                    }
                    if mask.condition(p).orient != moved.condition(p).orient {
                        mask.0[P::index(p)].orient = None;
                        changed = true;
                    }
                }
            }
        }
        let marked = mask
            .0
            .iter()
            .map(|s| SlotCondition {
                label: s.label.is_some().then_some(()),
                orient: s.orient,
            })
            .collect();
        Self(marked)
    }
}

#[cfg(test)]
mod test {
    use crate::Cube3x3;

    use super::*;

    #[test]
    fn applies_to_composes_correctly_on_full_cube() {
        let cube = Cube3x3::apply_scramble();
        let mask = Mask::<Cube3x3>::new_from_pieces(Cube3x3::ALL_PIECES.iter().copied());

        assert!(
            (mask.composed_with(&cube)).applies_to(&cube),
            "{}\n\n{}",
            mask.composed_with(&cube),
            cube
        );
    }
}
