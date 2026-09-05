use rubiks::{Cube3By3, Inv, Pow};

const IMPLEMENTED_MOVES: [&str; 8] = ["R", "U", "D", "L", "E", "S", "M", "y"];

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
fn sexy_move_has_correct_period() {
    for k in 1..6 {
        assert!(
            !Cube3By3::from_solved("R U R' U'").pow(k).is_solved(),
            "(R U R' U')^{k} should not be solved"
        );
    }
    assert!(
        Cube3By3::from_solved("R U R' U'").pow(6).is_solved(),
        "(R U R' U')^6 should be solved"
    );
}

#[test]
fn adjacent_face_sequence_has_constant_and_correct_period() {
    let pairs = [("R", "U"), ("U", "L"), ("L", "D"), ("D", "R")];
    let period = 105;
    for p in pairs {
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
fn slice_face_has_constand_and_correct_period() {
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
