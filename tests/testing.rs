// `?` reports setup failures; `assert!` reports the property under test failing.
#![allow(clippy::panic_in_result_fn)]

use rubiks_cube_lib::{Center, Corner, Cube3By3, Edge, Inv, Pow, zn::ZnRing};

const IMPLEMENTED_MOVES: [&str; 12] = ["R", "U", "D", "L", "F", "B", "E", "S", "M", "y", "z", "x"];

/// A reconstruction is a scramble followed by a solution. The scrambled state
/// must be reachable, and the final state must be reachable and solved.
fn assert_reconstruction(scramble: &str, solution: &str) -> Result<(), String> {
    let scrambled = Cube3By3::from_solved(scramble)?;
    assert!(
        scrambled.is_reachable(),
        "scramble reached an unreachable state: {scrambled:?}"
    );
    let finished = scrambled.move_sequence(solution)?;
    assert!(finished.is_reachable());
    assert!(finished.is_solved(), "not solved: {finished:?}");
    Ok(())
}

#[test]
fn default_is_reachable() {
    assert!(Cube3By3::default().is_reachable());
}

#[test]
fn r_is_reachable() -> Result<(), String> {
    assert!(Cube3By3::from_solved("R")?.is_reachable());
    Ok(())
}

#[test]
fn m_is_reachable() -> Result<(), String> {
    assert!(Cube3By3::from_solved("M")?.is_reachable());
    Ok(())
}

#[test]
fn y_is_reachable() -> Result<(), String> {
    assert!(Cube3By3::from_solved("y")?.is_reachable());
    Ok(())
}

// Handedness pins: one fact from the physical cube per base move, stated as
// "after the move, the piece now at slot X came from Y". These are the only
// tests that do not depend on how the move table derives one move from
// another, so they are the ones that catch a mirrored base move.

// Faces: clockwise when looking at that face.
#[test]
fn r_takes_front_top_corner_to_back_top() -> Result<(), String> {
    assert_eq!(
        Cube3By3::from_solved("R")?.corners().piece_at(Corner::Ubr),
        Corner::Ufr
    );
    Ok(())
}
#[test]
fn l_takes_back_top_corner_to_front_top() -> Result<(), String> {
    assert_eq!(
        Cube3By3::from_solved("L")?.corners().piece_at(Corner::Ufl),
        Corner::Ubl
    );
    Ok(())
}
#[test]
fn u_takes_front_right_corner_to_front_left() -> Result<(), String> {
    assert_eq!(
        Cube3By3::from_solved("U")?.corners().piece_at(Corner::Ufl),
        Corner::Ufr
    );
    Ok(())
}
#[test]
fn d_takes_front_right_corner_to_back_right() -> Result<(), String> {
    assert_eq!(
        Cube3By3::from_solved("D")?.corners().piece_at(Corner::Dbr),
        Corner::Dfr
    );
    Ok(())
}
#[test]
fn f_takes_top_right_corner_to_bottom_right() -> Result<(), String> {
    assert_eq!(
        Cube3By3::from_solved("F")?.corners().piece_at(Corner::Dfr),
        Corner::Ufr
    );
    Ok(())
}
#[test]
fn b_takes_top_right_corner_to_top_left() -> Result<(), String> {
    assert_eq!(
        Cube3By3::from_solved("B")?.corners().piece_at(Corner::Ubl),
        Corner::Ubr
    );
    Ok(())
}

// Slices: each follows its reference face, so it moves centers the way that face moves corners.

#[test]
fn e_follows_d_and_takes_front_center_to_right() -> Result<(), String> {
    assert_eq!(
        Cube3By3::from_solved("E")?.centers().piece_at(Center::R),
        Center::F
    );
    Ok(())
}
#[test]
fn m_follows_l_and_takes_top_center_to_front() -> Result<(), String> {
    assert_eq!(
        Cube3By3::from_solved("M")?.centers().piece_at(Center::F),
        Center::U
    );
    Ok(())
}
#[test]
fn s_follows_f_and_takes_top_center_to_right() -> Result<(), String> {
    assert_eq!(
        Cube3By3::from_solved("S")?.centers().piece_at(Center::R),
        Center::U
    );
    Ok(())
}

#[test]
fn r_move_respects_bounds_and_touches_only_r_layer() -> Result<(), String> {
    let r = Cube3By3::from_solved("R")?;
    for c in [Corner::Ubl, Corner::Ufl, Corner::Dfl, Corner::Dbl] {
        assert_eq!(r.corners().piece_at(c), c);
        assert_eq!(r.corners().orientation_at(c), ZnRing::ZERO);
    }
    for e in [
        Edge::Ub,
        Edge::Uf,
        Edge::Ul,
        Edge::Fl,
        Edge::Bl,
        Edge::Df,
        Edge::Db,
        Edge::Dl,
    ] {
        assert_eq!(r.edges().piece_at(e), e);
        assert_eq!(r.edges().orientation_at(e), ZnRing::ZERO);
    }
    assert!(r.is_reachable());
    Ok(())
}

#[test]
fn identity_is_two_sided() -> Result<(), String> {
    let r = Cube3By3::from_solved("R")?;
    assert_eq!(Cube3By3::default() * r, r);
    assert_eq!(r * Cube3By3::default(), r);
    Ok(())
}

#[test]
fn pow_0_gives_identity_cube() -> Result<(), String> {
    assert_eq!(
        Cube3By3::from_solved("R U R' U'")?.pow(0),
        Cube3By3::IDENTITY
    );
    Ok(())
}

#[test]
fn pow_finishes_correctly_for_large_exponent() -> Result<(), String> {
    let large_u32 = u32::MAX - 2;
    let r = Cube3By3::from_solved("R")?;
    assert_eq!(large_u32 % 4, 1);
    assert_eq!(r.pow(large_u32), r);
    Ok(())
}

#[test]
fn clockwise_moves_have_order_exactly_4() -> Result<(), String> {
    for m in &IMPLEMENTED_MOVES[..9] {
        let cube = Cube3By3::from_solved(m)?;
        for k in 1..4 {
            assert!(!cube.pow(k).is_solved(), "{m}^{k} should not be solved");
        }
        assert!(cube.pow(4).is_solved(), "{m}^4 should be solved");
    }
    Ok(())
}

#[test]
fn move_inverse_is_move_cubed() -> Result<(), String> {
    for m in IMPLEMENTED_MOVES {
        let cube = Cube3By3::from_solved(m)?;
        assert_eq!(cube.inverse(), cube.pow(3));
    }
    Ok(())
}

#[test]
fn inverse_is_an_involution() -> Result<(), String> {
    for m in IMPLEMENTED_MOVES {
        let cube = Cube3By3::from_solved(m)?;
        assert_eq!(cube.inverse().inverse(), cube);
    }
    assert_eq!(Cube3By3::default().inverse(), Cube3By3::default());
    Ok(())
}

#[test]
fn mul_is_associative() -> Result<(), String> {
    let a = Cube3By3::from_solved("R")?;
    let b = Cube3By3::from_solved("U2 L")?.pow(2);
    let c = Cube3By3::from_solved("y")?.inverse();
    assert_eq!((a * b) * c, a * (b * c));
    Ok(())
}

#[test]
fn inverse_of_product_reverses_order() -> Result<(), String> {
    let a = Cube3By3::from_solved("U")?.pow(1);
    let b = Cube3By3::from_solved("R")?.pow(2);
    assert_eq!((a * b).inverse(), b.inverse() * a.inverse());
    assert_ne!((a * b).inverse(), a.inverse() * b.inverse());
    Ok(())
}

#[test]
fn r_and_l_commute() -> Result<(), String> {
    assert!(Cube3By3::from_solved("R L R' L'")?.is_solved());
    Ok(())
}

#[test]
fn u_and_d_commute() -> Result<(), String> {
    assert!(Cube3By3::from_solved("U D U' D'")?.is_solved());
    Ok(())
}

#[test]
fn multiple_moves_break_down_correctly() -> Result<(), String> {
    assert_eq!(
        Cube3By3::from_solved("R U R' U'")?,
        Cube3By3::IDENTITY
            .move_sequence("R")?
            .move_sequence("U")?
            .move_sequence("R'")?
            .move_sequence("U'")?
    );
    Ok(())
}

#[test]
fn fmc_wr_as_multiplication() -> Result<(), String> {
    let scramble = Cube3By3::from_solved(
        "R' U' F D2 L2 F R2 U2 R2 B D2 L B2 D' B2 L' R' B D2 B U2 L U2 R' U' F",
    )?;
    let solve = Cube3By3::from_solved("    D2 F' D2 U2 F' L2 D R2 D B2 F L2 R' F' D U'")?;
    assert!(scramble.is_reachable());
    assert!((scramble * solve).is_solved(), "{:?}", scramble * solve);
    Ok(())
}

const CFOP_SCRAMBLE: &str = "R2 F' L2 D2 F2 U2 B' L2 F R2 D2 F2 D L' U B R' F' R D R2 U2 ";
const ROUX_SCRAMBLE: &str = "U' L2 D' B2 D R2 F2 D' B2 R2 D B' R F2 R D' B' F U2 R' U D ";

#[test]
fn cfop_solve_hygienized() -> Result<(), String> {
    assert_reconstruction(
        CFOP_SCRAMBLE,
        "z y2
            U' R' L2 x'
            F' U F
            R' U' R U R' U' R
            L U' L'
            U y' U R U' R' U' R U' R2 F R
            U R U' R' U R U2 R' U' R U R' F'",
    )
}

#[test]
fn cfop_solve() -> Result<(), String> {
    assert_reconstruction(
        CFOP_SCRAMBLE,
        "z y2
            U' R' L2 x'
            F' U F
            R' U' R U R' U' R
            L U' L'
            U y' U R U' R' U' R U' R2' F R
            U R U' R' U R U2' R' U' R U R' F'",
    )
}

#[test]
fn roux_solve_with_comments() -> Result<(), String> {
    assert_reconstruction(
        ROUX_SCRAMBLE,
        "y2 F' M F' R U' R U' Fw z' // FB
            U R U r M' U' R U2' R' // SS
            U R' U' R U' R' U' r // SP (CMLL skip)
            U M' U' M U' U' M' U M // EOLR
            U' U' M2' U' M U' U' M' U' U' M2' // EP",
    )
}

#[test]
fn roux_solve_without_comments() -> Result<(), String> {
    assert_reconstruction(
        ROUX_SCRAMBLE,
        "y2 F' M F' R U' R U' Fw z'
            U R U r M' U' R U2' R'
            U R' U' R U' R' U' r
            U M' U' M U' U' M' U M
            U' U' M2' U' M U' U' M' U' U' M2' ",
    )
}

#[test]
fn s_based_roux_solve() -> Result<(), String> {
    assert_reconstruction(
        ROUX_SCRAMBLE,
        "y2 F' M F' R U' R U' Fw z' // FB
U R U r M' U' R U2' R' // SS
U R' U' R U' R' U' r // SP (CMLL skip)
y // mean rotation
U S U' S' U2 S U S' U2 S2 U' // EOLR
 S' U2 S U2 S2 // 4c",
    )
}

#[test]
fn e_based_roux_solve() -> Result<(), String> {
    assert_reconstruction(
        ROUX_SCRAMBLE,
        "y2 F' M F' R U' R U' Fw z' // FB
U R U r M' U' R U2' R' // SS
U R' U' R U' R' U' r // SP (CMLL skip)
z // mean rotation
R E R' E' R2 E R E' R2 E2 R'   // EOLR
E' R2 E R2 E2// 4c",
    )
}

#[test]
fn roux_solve_without_wide_moves() -> Result<(), String> {
    assert_reconstruction(
        ROUX_SCRAMBLE,
        "y2 F' M F' R U' R U' B
            U R U R M2 U' R U2 R'
            U R' U' R U' R' U' R M'
            U M' U' M U' U' M' U M
            U' U' M2 U' M U' U' M' U' U' M2 ",
    )
}

#[test]
fn roux_solve_removes_comments() -> Result<(), String> {
    assert_eq!(
        Cube3By3::from_solved(concat!(
            "U' L2 D' B2 D R2 F2 D' B2 R2 D B' R F2 R D' B' F U2 R' U D ",
            "y2 F' M F' R U' R U' Fw z'
            U R U r M' U' R U2' R'
            U R' U' R U' R' U' r
            U M' U' M U' U' M' U M
            U' U' M2' U' M U' U' M' U' U' M2' "
        ))?,
        Cube3By3::from_solved(concat!(
            "U' L2 D' B2 D R2 F2 D' B2 R2 D B' R F2 R D' B' F U2 R' U D ",
            "y2 F' M F' R U' R U' Fw z' // FB
            U R U r M' U' R U2' R' // SS
            U R' U' R U' R' U' r // SP (CMLL skip)
            U M' U' M U' U' M' U M // EOLR
            U' U' M2' U' M U' U' M' U' U' M2' // EP"
        ))?
    );
    Ok(())
}

#[test]
fn sexy_move_has_correct_period_on_all_face_pairs() -> Result<(), String> {
    let adjacent_face_pairs = [
        ("R", "U"),
        ("U", "L"),
        ("L", "D"),
        ("D", "R"),
        ("F", "R"),
        ("F", "U"),
        ("F", "L"),
        ("F", "D"),
        ("B", "R"),
        ("B", "U"),
        ("B", "L"),
        ("B", "D"),
    ];

    let sexy: Vec<String> = adjacent_face_pairs
        .iter()
        .map(|&(m1, m2)| String::from(m1) + " " + m2 + " " + m1 + "' " + m2 + "' ")
        .collect();
    for s in sexy {
        let cube = Cube3By3::from_solved(&s)?;
        for k in 1..6 {
            assert!(!cube.pow(k).is_solved(), "({s})^{k} should not be solved");
        }
        assert!(cube.pow(6).is_solved(), "({s})^6 should be solved");
    }
    Ok(())
}

#[test]
fn adjacent_face_sequence_has_constant_and_correct_period() -> Result<(), String> {
    let adjacent_face_pairs = [
        ("R", "U"),
        ("U", "L"),
        ("L", "D"),
        ("D", "R"),
        ("F", "R"),
        ("F", "U"),
        ("F", "L"),
        ("F", "D"),
        ("B", "R"),
        ("B", "U"),
        ("B", "L"),
        ("B", "D"),
    ];
    let period = 105;
    for p in adjacent_face_pairs {
        let mut c = Cube3By3::IDENTITY;
        for _ in 1..period {
            c = c.move_sequence(p.0)?.move_sequence(p.1)?;
            assert!(
                !c.is_solved(),
                "the period hasn't arrived for {} {}",
                p.0,
                p.1
            );
        }
        c = c.move_sequence(p.0)?.move_sequence(p.1)?;
        assert!(
            c.is_solved(),
            "the period should've arrived for {} {}",
            p.0,
            p.1
        );
    }
    Ok(())
}

#[test]
fn slice_face_has_constant_and_correct_period() -> Result<(), String> {
    let pairs = [
        ("M", "U"),
        ("M", "F"),
        ("M", "D"),
        ("M", "B"),
        ("E", "F"),
        ("E", "R"),
        ("E", "B"),
        ("E", "L"),
        ("S", "U"),
        ("S", "R"),
        ("S", "D"),
        ("S", "L"),
    ];
    let period = 8;
    for p in pairs {
        let mut c = Cube3By3::IDENTITY;
        for i in 1..period {
            c = c.move_sequence(p.0)?.move_sequence(p.1)?;
            assert!(
                !c.is_solved(),
                "the period shouldn't have arrived for ({} {})^{i}",
                p.0,
                p.1
            );
        }
        c = c.move_sequence(p.0)?.move_sequence(p.1)?;
        assert!(
            c.is_solved(),
            "the period should've arrived for {} {}",
            p.0,
            p.1
        );
    }
    Ok(())
}

#[test]
fn random_words_stay_reachable_and_undo_cleanly() -> Result<(), String> {
    // Each pair is a modifier and its inverse; `2` and `2'` reach the same
    // state, so inverting a double is the other spelling of it.
    const MODIFIER_PAIRS: [(&str, &str); 2] = [("", "'"), ("2", "2'")];
    fastrand::seed(11);
    for _ in 0..50 {
        let len = fastrand::usize(1..30);
        let mut forward = Vec::with_capacity(len);
        let mut backward = Vec::with_capacity(len);
        for _ in 0..len {
            let part = IMPLEMENTED_MOVES[fastrand::usize(..IMPLEMENTED_MOVES.len())];
            let (modifier, inverse_modifier) = MODIFIER_PAIRS[fastrand::usize(..2)];
            let (modifier, inverse_modifier) = if fastrand::bool() {
                (inverse_modifier, modifier)
            } else {
                (modifier, inverse_modifier)
            };
            forward.push(format!("{part}{modifier}"));
            backward.push(format!("{part}{inverse_modifier}"));
        }
        backward.reverse();
        let forward = forward.join(" ");
        let backward = backward.join(" ");

        let mut cube = Cube3By3::default();
        for token in forward.split(' ') {
            cube = cube.move_sequence(token)?;
            assert!(cube.is_reachable(), "unreachable somewhere in {forward}");
        }
        assert_eq!(Cube3By3::from_solved(&forward)?, cube);
        assert!(
            cube.move_sequence(&backward)?.is_solved(),
            "{forward} then {backward} should be solved"
        );
        assert_eq!(cube * cube.inverse(), Cube3By3::default());
    }
    Ok(())
}
