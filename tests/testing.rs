use rubiks::{Cube3By3, Inv, Pow};

#[test]
fn default_respects_parity() {
    assert!(Cube3By3::default().respects_orientation_parity());
}

#[test]
fn one_r_is_not_solved() {
    assert!(!(Cube3By3::default() * Cube3By3::R).is_solved());
}

#[test]
fn identity_is_two_sided() {
    assert_eq!(Cube3By3::default() * Cube3By3::R, Cube3By3::R);
    assert_eq!(Cube3By3::R * Cube3By3::default(), Cube3By3::R);
}

#[test]
fn r_has_order_exactly_4() {
    for k in 1..4 {
        assert!(
            !Cube3By3::R.pow(k).is_solved(),
            "R^{k} should not be solved"
        );
    }
    assert!(Cube3By3::R.pow(4).is_solved());
}

#[test]
fn r_inverse_is_r_cubed() {
    assert_eq!(Cube3By3::R.inverse(), Cube3By3::R.pow(3));
}

#[test]
fn inverse_is_an_involution() {
    assert_eq!(Cube3By3::R.inverse().inverse(), Cube3By3::R);
    assert_eq!(Cube3By3::default().inverse(), Cube3By3::default());
}

#[test]
fn mul_is_associative() {
    let a = Cube3By3::R;
    let b = Cube3By3::R.pow(2);
    let c = Cube3By3::R.inverse();
    assert_eq!((a.clone() * b.clone()) * c.clone(), a * (b * c));
}

#[test]
fn inverse_of_product_reverses_order() {
    let a = Cube3By3::R.pow(1);
    let b = Cube3By3::R.pow(2);
    assert_eq!((a.clone() * b.clone()).inverse(), b.inverse() * a.inverse());
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
                Cube3By3::R.inverse()
            } else {
                Cube3By3::R
            };
            cube = cube * m;
            assert!(cube.respects_orientation_parity());
        }

        let mut undo = cube.clone();
        for &prime in word.iter().rev() {
            let m = if prime {
                Cube3By3::R
            } else {
                Cube3By3::R.inverse()
            };
            undo = undo * m;
        }
        assert!(undo.is_solved());
        assert_eq!(cube.clone() * cube.inverse(), Cube3By3::default());
    }
}
