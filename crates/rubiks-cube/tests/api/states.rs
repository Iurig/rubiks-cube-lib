use rubiks_cube::*;
use std::error::Error;

#[test]
fn default_is_reachable() {
    assert!(Cube3x3::default().is_reachable());
}

#[test]
fn r_is_reachable() -> Result<(), Box<dyn Error>> {
    assert!(Cube3x3::from_solved("R")?.is_reachable());
    Ok(())
}

#[test]
fn m_is_reachable() -> Result<(), Box<dyn Error>> {
    assert!(Cube3x3::from_solved("M")?.is_reachable());
    Ok(())
}

#[test]
fn y_is_reachable() -> Result<(), Box<dyn Error>> {
    assert!(Cube3x3::from_solved("y")?.is_reachable());
    Ok(())
}

#[test]
fn hundred_random_states_are_reachable() {
    for seed in 0..100 {
        assert!(
            Cube3x3::random_state_with_seed(seed).is_reachable(),
            "the state from seed {seed} is not reachable"
        );
    }
}

#[test]
fn default_is_solved() {
    assert!(Cube3x3::default().is_solved());
}

#[test]
fn y_rotated_solved_is_solved() -> Result<(), Box<dyn Error>> {
    let rotated_def = Cube3x3::from_solved("y")?;
    assert!(rotated_def.is_solved());
    Ok(())
}

#[test]
fn rotated_solved_is_solved() -> Result<(), Box<dyn Error>> {
    let rotated_def = Cube3x3::from_solved("y z y z x2 z2")?;
    assert!(rotated_def.is_solved());
    Ok(())
}

#[test]
fn all_axes_rotated_solved_is_solved_but_not_after_a_face_turn() -> Result<(), Box<dyn Error>> {
    assert!(Cube3x3::from_solved("x y2 z'")?.is_solved());
    assert!(!Cube3x3::from_solved("x y2 z' R")?.is_solved());
    Ok(())
}
