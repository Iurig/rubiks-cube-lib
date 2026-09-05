use rubiks::{Cube3By3, Inv, Pow};

#[test]
fn default_respects_parity() {
    assert!(Cube3By3::default().respects_orientation_parity());
}

#[test]
fn one_r_is_not_solved() {
    assert!(!(Cube3By3::default() * Cube3By3::from_solved("R")).is_solved());
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
    let moves = ["R", "U", "D", "L", "E", "y"];
    for m in moves {
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
fn r_inverse_is_r_cubed() {
    assert_eq!(
        Cube3By3::from_solved("R").inverse(),
        Cube3By3::from_solved("R").pow(3)
    );
}

#[test]
fn inverse_is_an_involution() {
    assert_eq!(
        Cube3By3::from_solved("R").inverse().inverse(),
        Cube3By3::from_solved("R")
    );
    assert_eq!(Cube3By3::default().inverse(), Cube3By3::default());
}

#[test]
fn mul_is_associative() {
    let a = Cube3By3::from_solved("R");
    let b = Cube3By3::from_solved("R").pow(2);
    let c = Cube3By3::from_solved("R").inverse();
    assert_eq!((a * b) * c, a * (b * c));
}

#[test]
fn inverse_of_product_reverses_order() {
    let a = Cube3By3::from_solved("R").pow(1);
    let b = Cube3By3::from_solved("R").pow(2);
    assert_eq!((a * b).inverse(), b.inverse() * a.inverse());
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
