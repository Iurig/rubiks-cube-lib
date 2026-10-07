use rubiks_cube::{zn::Zn, *};
use std::error::Error;

// Handedness pins: one fact from the physical cube per base move, stated as
// "after the move, the piece now at slot X came from Y". These are the only
// tests that do not depend on how the move table derives one move from
// another, so they are the ones that catch a mirrored base move.

// Faces: clockwise when looking at that face.
#[test]
fn r_takes_front_top_corner_to_back_top() -> Result<(), Box<dyn Error>> {
    assert_eq!(
        Cube3x3::from_solved("R")?.corners().piece_at(Corner::Ubr),
        Corner::Ufr
    );
    Ok(())
}
#[test]
fn l_takes_back_top_corner_to_front_top() -> Result<(), Box<dyn Error>> {
    assert_eq!(
        Cube3x3::from_solved("L")?.corners().piece_at(Corner::Ufl),
        Corner::Ubl
    );
    Ok(())
}
#[test]
fn u_takes_front_right_corner_to_front_left() -> Result<(), Box<dyn Error>> {
    assert_eq!(
        Cube3x3::from_solved("U")?.corners().piece_at(Corner::Ufl),
        Corner::Ufr
    );
    Ok(())
}
#[test]
fn d_takes_front_right_corner_to_back_right() -> Result<(), Box<dyn Error>> {
    assert_eq!(
        Cube3x3::from_solved("D")?.corners().piece_at(Corner::Dbr),
        Corner::Dfr
    );
    Ok(())
}
#[test]
fn f_takes_top_right_corner_to_bottom_right() -> Result<(), Box<dyn Error>> {
    assert_eq!(
        Cube3x3::from_solved("F")?.corners().piece_at(Corner::Dfr),
        Corner::Ufr
    );
    Ok(())
}
#[test]
fn b_takes_top_right_corner_to_top_left() -> Result<(), Box<dyn Error>> {
    assert_eq!(
        Cube3x3::from_solved("B")?.corners().piece_at(Corner::Ubl),
        Corner::Ubr
    );
    Ok(())
}

// Slices: each follows its reference face, so it moves centers the way that face moves corners.

#[test]
fn e_follows_d_and_takes_front_center_to_right() -> Result<(), Box<dyn Error>> {
    assert_eq!(
        Cube3x3::from_solved("E")?.centers().piece_at(Center::R),
        Center::F
    );
    Ok(())
}
#[test]
fn m_follows_l_and_takes_top_center_to_front() -> Result<(), Box<dyn Error>> {
    assert_eq!(
        Cube3x3::from_solved("M")?.centers().piece_at(Center::F),
        Center::U
    );
    Ok(())
}
#[test]
fn s_follows_f_and_takes_top_center_to_right() -> Result<(), Box<dyn Error>> {
    assert_eq!(
        Cube3x3::from_solved("S")?.centers().piece_at(Center::R),
        Center::U
    );
    Ok(())
}

#[test]
fn r_move_respects_bounds_and_touches_only_r_layer() -> Result<(), Box<dyn Error>> {
    let r = Cube3x3::from_solved("R")?;
    for c in [Corner::Ubl, Corner::Ufl, Corner::Dfl, Corner::Dbl] {
        assert_eq!(r.corners().piece_at(c), c);
        assert_eq!(r.corners().orientation_at(c), Zn::ZERO);
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
        assert_eq!(r.edges().orientation_at(e), Zn::ZERO);
    }
    assert!(r.is_reachable());
    Ok(())
}

#[test]
fn r_and_l_commute() -> Result<(), Box<dyn Error>> {
    assert!(Cube3x3::from_solved("R L R' L'")?.is_solved());
    Ok(())
}

#[test]
fn u_and_d_commute() -> Result<(), Box<dyn Error>> {
    assert!(Cube3x3::from_solved("U D U' D'")?.is_solved());
    Ok(())
}

#[test]
fn rotations_match_moves() -> Result<(), Box<dyn Error>> {
    assert_eq!(Cube3x3::from_solved("y")?, Cube3x3::from_solved("U E' D'")?);
    assert_eq!(Cube3x3::from_solved("z")?, Cube3x3::from_solved("F S B'")?);
    assert_eq!(Cube3x3::from_solved("x")?, Cube3x3::from_solved("R M' L'")?);
    Ok(())
}

#[test]
fn r_2_is_equal_to_r_prime_2() -> Result<(), Box<dyn Error>> {
    let r2 = Cube3x3::from_solved("R2")?;
    let r_prime_2 = Cube3x3::from_solved("R' R'")?;
    assert_eq!(r2, r_prime_2);
    Ok(())
}

#[test]
fn sexy_move_has_correct_period_on_all_face_pairs() -> Result<(), Box<dyn Error>> {
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
        let cube = Cube3x3::from_solved(&s)?;
        for k in 1..6 {
            assert!(!cube.pow(k).is_solved(), "({s})^{k} should not be solved");
        }
        assert!(cube.pow(6).is_solved(), "({s})^6 should be solved");
    }
    Ok(())
}

#[test]
fn adjacent_face_sequence_has_constant_and_correct_period() -> Result<(), Box<dyn Error>> {
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
        let mut c = Cube3x3::default();
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
fn slice_face_has_constant_and_correct_period() -> Result<(), Box<dyn Error>> {
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
        let mut c = Cube3x3::default();
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
