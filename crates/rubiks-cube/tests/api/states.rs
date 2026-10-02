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
