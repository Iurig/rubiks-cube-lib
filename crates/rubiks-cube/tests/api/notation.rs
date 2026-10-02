use rubiks_cube::*;
use std::error::Error;

#[test]
fn multiple_moves_break_down_correctly() -> Result<(), Box<dyn Error>> {
    assert_eq!(
        Cube3x3::from_solved("R U R' U'")?,
        Cube3x3::default()
            .move_sequence("R")?
            .move_sequence("U")?
            .move_sequence("R'")?
            .move_sequence("U'")?
    );
    Ok(())
}

#[test]
fn bad_move_error_names_its_line_and_position() {
    let e = Cube3x3::from_solved(
        "R U R'
    F U F'
    Mw' M",
    )
    .unwrap_err();

    assert_eq!(e.line(), 3);
    assert_eq!(e.position(), 1);
    assert_eq!(
        *e.cause(),
        rubiks_cube::ParseMoveError::BadModifier {
            invalid_move: "Mw'".to_string(),
            modifier: "w'".to_string(),
        }
    );
}
