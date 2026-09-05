use rubiks::{Cube3By3, Inv, Pow};

const IMPLEMENTED_MOVES: [&str; 10] = ["R", "U", "D", "L", "F", "B", "E", "S", "M", "y"];

#[test]
fn default_respects_parity() {
    assert!(Cube3By3::default().respects_orientation_parity());
}

#[test]
fn identity_is_two_sided() {
    assert_eq!(
        Cube3By3::default() * Cube3By3::from_solved("R"),
        Cube3By3::from_solved("R")
    );
    assert_eq!(
        Cube3By3::from_solved("R") * Cube3By3::default(),
        Cube3By3::from_solved("R")
    );
}

#[test]
fn clockwise_moves_have_order_exactly_4() {
    for m in IMPLEMENTED_MOVES {
        for k in 1..4 {
            assert!(
                !Cube3By3::from_solved(m).pow(k).is_solved(),
                "{m}^{k} should not be solved"
            );
        }
        assert!(
            Cube3By3::from_solved(m).pow(4).is_solved(),
            "{m}^4 should be solved"
        );
    }
}

#[test]
fn move_inverse_is_move_cubed() {
    for m in IMPLEMENTED_MOVES {
        assert_eq!(
            Cube3By3::from_solved(m).inverse(),
            Cube3By3::from_solved(m).pow(3)
        );
    }
}

#[test]
fn inverse_is_an_involution() {
    for m in IMPLEMENTED_MOVES {
        assert_eq!(
            Cube3By3::from_solved(m).inverse().inverse(),
            Cube3By3::from_solved(m)
        );
    }
    assert_eq!(Cube3By3::default().inverse(), Cube3By3::default());
}

#[test]
fn mul_is_associative() {
    let a = Cube3By3::from_solved("R");
    let b = Cube3By3::from_solved("U2 L").pow(2);
    let c = Cube3By3::from_solved("y").inverse();
    assert_eq!((a * b) * c, a * (b * c));
}

#[test]
fn inverse_of_product_reverses_order() {
    let a = Cube3By3::from_solved("U").pow(1);
    let b = Cube3By3::from_solved("R").pow(2);
    assert_eq!((a * b).inverse(), b.inverse() * a.inverse());
    assert_ne!((a * b).inverse(), a.inverse() * b.inverse());
}

#[test]
fn r_and_l_commute() {
    assert!(Cube3By3::from_solved("R L R' L'").is_solved());
}

#[test]
fn u_and_d_commute() {
    assert!(Cube3By3::from_solved("U D U' D'").is_solved());
}

#[test]
fn multiple_moves_break_down_correctly() {
    assert_eq!(
        Cube3By3::from_solved("R U R' U'"),
        Cube3By3::IDENTITY
            .move_sequence("R")
            .move_sequence("U")
            .move_sequence("R'")
            .move_sequence("U'")
    );
}

#[test]
fn fmc_wr_as_multiplication() {
    let scramble = Cube3By3::from_solved(
        "R' U' F D2 L2 F R2 U2 R2 B D2 L B2 D' B2 L' R' B D2 B U2 L U2 R' U' F",
    );
    let solve = Cube3By3::from_solved("    D2 F' D2 U2 F' L2 D R2 D B2 F L2 R' F' D U'");
    assert!((scramble * solve).is_solved(), "{:?}", scramble * solve);
}

#[test]
#[ignore = "rotations not yet implemented"]
fn cfop_solve() {
    let scramble = "R2 F' L2 D2 F2 U2 B' L2 F R2 D2 F2 D L' U B R' F' R D R2 U2 ";
    let solve = "z y2 
            U' R' L2 x' 
            F' U F 
            R' U' R U R' U' R 
            L U' L' 
            U y' U R U' R' U' R U' R2' F R 
            U R U' R' U R U2' R' U' R U R' F'";
    assert!(
        Cube3By3::IDENTITY
            .move_sequence(scramble)
            .move_sequence(solve)
            .is_solved()
    );
}

#[test]
#[ignore = "rotations AND wide moves not yet implemented"]
fn roux_solve_with_comments() {
    assert!(
        Cube3By3::from_solved(concat!(
            "U' L2 D' B2 D R2 F2 D' B2 R2 D B' R F2 R D' B' F U2 R' U D ",
            "y2 F' M F' R U' R U' Fw z' // FB
            U R U r M' U' R U2' R' // SS
            U R' U' R U' R' U' r // SP (CMLL skip)
            U M' U' M U' U' M' U M // EOLR
            U' U' M2' U' M U' U' M' U' U' M2' // EP"
        ))
        .is_solved()
    );
}

#[test]
#[ignore = "rotations AND wide moves not yet implemented"]
fn roux_solve_without_comments() {
    assert!(
        Cube3By3::from_solved(concat!(
            "U' L2 D' B2 D R2 F2 D' B2 R2 D B' R F2 R D' B' F U2 R' U D ",
            "y2 F' M F' R U' R U' Fw z' 
            U R U r M' U' R U2' R' 
            U R' U' R U' R' U' r 
            U M' U' M U' U' M' U M 
            U' U' M2' U' M U' U' M' U' U' M2' "
        ))
        .is_solved()
    );
}

#[test]
#[ignore = "comments are removed but roux solve lacks rotation and wide move implementation"]
fn roux_solve_removes_comments() {
    assert_eq!(
        Cube3By3::from_solved(concat!(
            "U' L2 D' B2 D R2 F2 D' B2 R2 D B' R F2 R D' B' F U2 R' U D ",
            "y2 F' M F' R U' R U' Fw z' 
            U R U r M' U' R U2' R' 
            U R' U' R U' R' U' r 
            U M' U' M U' U' M' U M 
            U' U' M2' U' M U' U' M' U' U' M2' "
        )),
        Cube3By3::from_solved(concat!(
            "U' L2 D' B2 D R2 F2 D' B2 R2 D B' R F2 R D' B' F U2 R' U D ",
            "y2 F' M F' R U' R U' Fw z' // FB
            U R U r M' U' R U2' R' // SS
            U R' U' R U' R' U' r // SP (CMLL skip)
            U M' U' M U' U' M' U M // EOLR
            U' U' M2' U' M U' U' M' U' U' M2' // EP"
        ))
    );
}

#[test]
fn sexy_move_has_correct_period_on_all_face_pairs() {
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
        for k in 1..6 {
            assert!(
                !Cube3By3::from_solved(&s).pow(k).is_solved(),
                "({s})^{k} should not be solved"
            );
        }
        assert!(
            Cube3By3::from_solved(&s).pow(6).is_solved(),
            "({s})^6 should be solved"
        );
    }
}

#[test]
fn adjacent_face_sequence_has_constant_and_correct_period() {
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
            c = c.move_sequence(p.0).move_sequence(p.1);
            assert!(
                !c.is_solved(),
                "the period hasn't arrived for {} {}",
                p.0,
                p.1
            );
        }
        c = c.move_sequence(p.0).move_sequence(p.1);
        assert!(
            c.is_solved(),
            "the period should've arrived for {} {}",
            p.0,
            p.1
        );
    }
}

#[test]
#[ignore = "known incorrect implementation of slices"]
fn slice_face_has_constant_and_correct_period() {
    let pairs = [
        ("M", "U"),
        ("M", "F"),
        ("M", "D"),
        ("M", "B"),
        ("S", "U"),
        ("S", "R"),
        ("S", "D"),
        ("S", "L"),
        ("E", "F"),
        ("E", "R"),
        ("E", "B"),
        ("E", "L"),
    ];
    let period = 12;
    for p in pairs {
        let mut c = Cube3By3::IDENTITY;
        for i in 1..period {
            c = c.move_sequence(p.0).move_sequence(p.1);
            assert!(
                !c.is_solved(),
                "the period hasn't arrived for ({} {})^{i}",
                p.0,
                p.1
            );
        }
        c = c.move_sequence(p.0).move_sequence(p.1);
        assert!(
            c.is_solved(),
            "the period should've arrived for {} {}",
            p.0,
            p.1
        );
    }
}

#[test]
fn random_r_words_keep_parity_and_undo_cleanly() {
    fastrand::seed(7);
    for _ in 0..50 {
        let len = fastrand::usize(1..30);
        let word: Vec<bool> = (0..len).map(|_| fastrand::bool()).collect();

        let mut cube = Cube3By3::default();
        for &prime in &word {
            let m = if prime {
                Cube3By3::from_solved("R").inverse()
            } else {
                Cube3By3::from_solved("R")
            };
            cube = cube * m;
            assert!(cube.respects_orientation_parity());
        }

        let mut undo = cube;
        for &prime in word.iter().rev() {
            let m = if prime {
                Cube3By3::from_solved("R")
            } else {
                Cube3By3::from_solved("R").inverse()
            };
            undo = undo * m;
        }
        assert!(undo.is_solved());
        assert_eq!(cube * cube.inverse(), Cube3By3::default());
    }
}
