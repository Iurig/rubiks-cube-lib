use std::{
    marker::PhantomData,
    sync::{LazyLock, Mutex},
};

use crate::{
    Cube3x3, Mask, NamedMoveSequences, Puzzle, SimpleStep, SolveMethod, SolveStep,
    methods::search::BFSMemo,
    puzzles::cube3by3::{
        moves::{MovablePart, Move3x3},
        pieces::{Faces, Pieces3x3, Slices},
    },
};

#[derive(Clone, Default, Debug)]
pub struct Roux(RouxOptions);

impl Roux {
    const FB_FRONT_SQUARE_PIECES: [Pieces3x3; 4] = [
        Pieces3x3::Center(crate::Center::L),
        Pieces3x3::Corner(crate::Corner::Dfl),
        Pieces3x3::Edge(crate::Edge::Fl),
        Pieces3x3::Edge(crate::Edge::Dl),
    ];

    const FB_BACK_SQUARE_PIECES: [Pieces3x3; 4] = [
        Pieces3x3::Center(crate::Center::L),
        Pieces3x3::Corner(crate::Corner::Dbl),
        Pieces3x3::Edge(crate::Edge::Bl),
        Pieces3x3::Edge(crate::Edge::Dl),
    ];

    const FB_PIECES: [Pieces3x3; 6] = [
        Pieces3x3::Center(crate::Center::L),
        Pieces3x3::Corner(crate::Corner::Dfl),
        Pieces3x3::Corner(crate::Corner::Dbl),
        Pieces3x3::Edge(crate::Edge::Fl),
        Pieces3x3::Edge(crate::Edge::Dl),
        Pieces3x3::Edge(crate::Edge::Bl),
    ];

    const SB_SQUARE_PIECES: [Pieces3x3; 4] = [
        Pieces3x3::Center(crate::Center::R),
        Pieces3x3::Edge(crate::Edge::Dr),
        Pieces3x3::Corner(crate::Corner::Dbr),
        Pieces3x3::Edge(crate::Edge::Br),
    ];

    const SB_PIECES: [Pieces3x3; 6] = [
        Pieces3x3::Center(crate::Center::R),
        Pieces3x3::Edge(crate::Edge::Dr),
        Pieces3x3::Corner(crate::Corner::Dbr),
        Pieces3x3::Edge(crate::Edge::Br),
        Pieces3x3::Edge(crate::Edge::Fr),
        Pieces3x3::Corner(crate::Corner::Dfr),
    ];

    const CMLL_PIECES: [Pieces3x3; 4] = [
        Pieces3x3::Corner(crate::Corner::Ufr),
        Pieces3x3::Corner(crate::Corner::Ufl),
        Pieces3x3::Corner(crate::Corner::Ubr),
        Pieces3x3::Corner(crate::Corner::Ubl),
    ];

    const LSE_PIECES: [Pieces3x3; 10] = [
        Pieces3x3::Edge(crate::Edge::Ub),
        Pieces3x3::Edge(crate::Edge::Ur),
        Pieces3x3::Edge(crate::Edge::Ul),
        Pieces3x3::Edge(crate::Edge::Uf),
        Pieces3x3::Edge(crate::Edge::Df),
        Pieces3x3::Edge(crate::Edge::Db),
        Pieces3x3::Center(crate::Center::U),
        Pieces3x3::Center(crate::Center::F),
        Pieces3x3::Center(crate::Center::D),
        Pieces3x3::Center(crate::Center::B),
    ];
}

/// The Roux steps in solving order, built once on first use.
static STEPS: LazyLock<Vec<SimpleStep<Cube3x3, Roux>>> = LazyLock::new(roux_steps);

/// Builds the step chain: each step's `before` is the previous step's `after`.
#[allow(clippy::too_many_lines)]
fn roux_steps() -> Vec<SimpleStep<Cube3x3, Roux>> {
    let fb_front_square_after = Mask::<Cube3x3>::new_from_pieces(Roux::FB_FRONT_SQUARE_PIECES);
    let fb_front_square = SimpleStep::<Cube3x3, Roux> {
        name: "FB Square".to_string(),
        before: Mask::<Cube3x3>::default(),
        after: fb_front_square_after.clone(),
        allowed_moves: Cube3x3::ALL_MOVES
            .iter()
            .map(|&m| (vec![m], true))
            .collect(),
        is_allowed: |m| !m.0.fb_as_one_step,
        memo: Mutex::new(BFSMemo::new(&fb_front_square_after)),
        phantom: PhantomData,
    };

    let fb_back_square_after = Mask::<Cube3x3>::new_from_pieces(Roux::FB_BACK_SQUARE_PIECES);
    let fb_back_square = SimpleStep::<Cube3x3, Roux> {
        name: "FB Square".to_string(),
        before: Mask::<Cube3x3>::default(),
        after: fb_back_square_after.clone(),
        allowed_moves: Cube3x3::ALL_MOVES
            .iter()
            .map(|&m| (vec![m], true))
            .collect(),
        is_allowed: |m| !m.0.fb_as_one_step,
        memo: Mutex::new(BFSMemo::new(&fb_back_square_after)),
        phantom: PhantomData,
    };

    let fb_front_pair_after = Mask::<Cube3x3>::new_from_pieces(Roux::FB_PIECES);
    let fb_front_pair = SimpleStep::<Cube3x3, Roux> {
        name: "FB Pair".to_string(),
        before: Mask::<Cube3x3>::new_from_pieces(Roux::FB_BACK_SQUARE_PIECES),
        after: fb_front_pair_after.clone(),
        allowed_moves: Cube3x3::ALL_MOVES
            .iter()
            .map(|&m| (vec![m], true))
            .collect(),
        is_allowed: |m| !m.0.fb_as_one_step,
        memo: Mutex::new(BFSMemo::new(&fb_front_pair_after)),
        phantom: PhantomData,
    };

    let fb_back_pair_after = Mask::<Cube3x3>::new_from_pieces(Roux::FB_PIECES);
    let fb_back_pair = SimpleStep::<Cube3x3, Roux> {
        name: "FB Pair".to_string(),
        before: Mask::<Cube3x3>::new_from_pieces(Roux::FB_FRONT_SQUARE_PIECES),
        after: fb_back_pair_after.clone(),
        allowed_moves: Cube3x3::ALL_MOVES
            .iter()
            .map(|&m| (vec![m], true))
            .collect(),
        is_allowed: |m| !m.0.fb_as_one_step,
        memo: Mutex::new(BFSMemo::new(&fb_back_pair_after)),
        phantom: PhantomData,
    };

    let fb_after = Mask::<Cube3x3>::new_from_pieces(Roux::FB_PIECES);
    let fb = SimpleStep::<Cube3x3, Roux> {
        name: "FB".to_string(),
        before: Mask::<Cube3x3>::new_from_pieces([]),
        after: fb_after.clone(),
        allowed_moves: Cube3x3::ALL_MOVES
            .iter()
            .map(|&m| (vec![m], true))
            .collect(),
        is_allowed: |_| true,
        memo: Mutex::new(BFSMemo::new(&fb_after)),
        phantom: PhantomData,
    };

    let second_block_moves = [
        MovablePart::Face(Faces::U),
        MovablePart::Face(Faces::R),
        MovablePart::Slice(Slices::M),
        MovablePart::Wide(Faces::R),
    ];

    let sb_square_after = Mask::<Cube3x3>::new_from_pieces(
        Roux::FB_PIECES
            .iter()
            .chain(Roux::SB_SQUARE_PIECES.iter())
            .copied(),
    );
    let sb_square = SimpleStep::<Cube3x3, Roux> {
        name: "SB Square".to_string(),
        before: fb.after.clone(),
        after: sb_square_after.clone(),
        allowed_moves: Cube3x3::ALL_MOVES
            .iter()
            .filter(|&m| second_block_moves.contains(&m.part))
            .map(|&m| (vec![m], true))
            .collect(),
        is_allowed: |m| !m.0.sb_square_as_one_step,
        memo: Mutex::new(BFSMemo::new(&sb_square_after)),
        phantom: PhantomData,
    };

    let sb_pair_after = Mask::<Cube3x3>::new_from_pieces(
        Roux::FB_PIECES
            .iter()
            .chain(Roux::SB_SQUARE_PIECES.iter())
            .chain(Roux::SB_PIECES.iter())
            .copied(),
    );
    let sb_pair = SimpleStep::<Cube3x3, Roux> {
        name: "SB Pair".to_string(),
        before: sb_square.after.clone(),
        after: sb_pair_after.clone(),
        allowed_moves: Cube3x3::ALL_MOVES
            .iter()
            .filter(|&m| second_block_moves.contains(&m.part))
            .map(|&m| (vec![m], true))
            .collect(),
        is_allowed: |m| !m.0.sb_square_as_one_step,
        memo: Mutex::new(BFSMemo::new(&sb_pair_after)),
        phantom: PhantomData,
    };

    let sb_after = Mask::<Cube3x3>::new_from_pieces(
        Roux::FB_PIECES
            .iter()
            .chain(Roux::SB_PIECES.iter())
            .copied(),
    );
    let sb = SimpleStep::<Cube3x3, Roux> {
        name: "SB".to_string(),
        before: fb.after.clone(),
        after: sb_after.clone(),
        allowed_moves: Cube3x3::ALL_MOVES
            .iter()
            .filter(|&m| second_block_moves.contains(&m.part))
            .map(|&m| (vec![m], true))
            .collect(),
        is_allowed: |m| m.0.sb_square_as_one_step,
        memo: Mutex::new(BFSMemo::new(&sb_after)),
        phantom: PhantomData,
    };

    let cmll_after = Mask::<Cube3x3>::new_from_pieces(
        Roux::FB_PIECES
            .iter()
            .chain(Roux::SB_PIECES.iter())
            .chain(Roux::CMLL_PIECES.iter())
            .copied(),
    );
    let cmll = SimpleStep::<Cube3x3, Roux> {
        name: "CMLL".to_string(),
        before: sb.after.clone(),
        after: cmll_after.clone(),
        allowed_moves: [
            ("R U R' U R U2 R'", true),
            ("R U2 R' U' R U' R'", true),
            ("R U R' F' R U R' U' R' F R2 U' R'", true),
            ("U", false),
            ("U2", false),
            ("U'", false),
            ("F R U' R' U' R U R' F' R U R' U' R' F R F'", true),
        ]
        .iter()
        .map(|(r, has_cost)| {
            (
                Move3x3::sequence(r)
                    .map(|p| p.expect("manually curated sequences should always parse"))
                    .collect(),
                *has_cost,
            )
        })
        .collect(),
        is_allowed: |_| true,
        memo: Mutex::new(BFSMemo::new(&cmll_after)),
        phantom: PhantomData,
    };

    let lse_after = Mask::<Cube3x3>::new_from_pieces(
        Roux::FB_PIECES
            .iter()
            .chain(Roux::SB_PIECES.iter())
            .chain(Roux::CMLL_PIECES.iter())
            .chain(Roux::LSE_PIECES.iter())
            .copied(),
    );
    let lse = SimpleStep::<Cube3x3, Roux> {
        name: "LSE".to_string(),
        before: cmll.after.clone(),
        after: lse_after.clone(),
        allowed_moves: Cube3x3::ALL_MOVES
            .iter()
            .filter(|&m| {
                [MovablePart::Face(Faces::U), MovablePart::Slice(Slices::M)].contains(&m.part)
            })
            .map(|&m| (vec![m], true))
            .collect(),
        is_allowed: |_| true,
        memo: Mutex::new(BFSMemo::new(&lse_after)),
        phantom: PhantomData,
    };

    vec![
        fb_front_square,
        fb_back_square,
        fb_front_pair,
        fb_back_pair,
        fb,
        sb_square,
        sb_pair,
        sb,
        cmll,
        lse,
    ]
}

#[derive(Clone, Debug)]
pub struct RouxOptions {
    pub sb_square_as_one_step: bool,
    pub fb_as_one_step: bool,
    pub one_look_cmll: bool,
}

impl Default for RouxOptions {
    fn default() -> Self {
        Self {
            sb_square_as_one_step: true,
            fb_as_one_step: true,
            one_look_cmll: true,
        }
    }
}

impl SolveMethod<Cube3x3, NamedMoveSequences<Cube3x3>> for Roux {
    type MethodOptions = RouxOptions;

    fn name(&self) -> String {
        "Roux".to_string()
    }

    fn from_options(options: Self::MethodOptions) -> Self {
        Self(options)
    }

    fn to_options(&self) -> &Self::MethodOptions {
        &self.0
    }

    fn steps(
        &self,
    ) -> Vec<&impl SolveStep<Cube3x3, Vec<(String, Vec<<Cube3x3 as Puzzle>::Moves>)>, Self>> {
        STEPS.iter().collect()
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use super::*;
    use crate::{Cube3x3, Inv, Puzzle};

    #[test]
    fn all_step_pieces_are_all_cube_pieces() {
        assert_eq!(
            Roux::FB_PIECES
                .iter()
                .chain(Roux::SB_SQUARE_PIECES.iter())
                .chain(Roux::SB_PIECES.iter())
                .chain(Roux::CMLL_PIECES.iter())
                .chain(Roux::LSE_PIECES.iter())
                .copied()
                .collect::<HashSet<Pieces3x3>>(),
            HashSet::from_iter(Cube3x3::ALL_PIECES.iter().copied())
        );
    }

    #[test]
    fn cmll_step_leaves_the_cube_with_cmll_solved() {
        let cmll = STEPS.iter().find(|s| s.name == "CMLL").unwrap();
        let sune = Cube3x3::from_solved("R U R' U R U2 R'").unwrap();
        for scramble in [Cube3x3::from_solved("U2").unwrap(), sune.inverse()] {
            let mut cube = scramble;
            cmll.solve(&mut cube).unwrap();
            assert!(cmll.step_is_solved(&cube), "not solved:\n{cube}");
        }
    }

    #[test]
    #[ignore = "reason"]
    fn step_requirements_meet_bound_conditions() {}
}
