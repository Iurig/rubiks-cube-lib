use rubiks_cube::*;

#[test]
fn mask_ignores_duplicates() {
    let pieces1 = [
        Pieces3x3::Center(Center::F),
        Pieces3x3::Center(Center::U),
        Pieces3x3::Edge(Edge::Uf),
    ];
    let pieces2 = [
        Pieces3x3::Center(Center::F),
        Pieces3x3::Center(Center::F),
        Pieces3x3::Center(Center::U),
        Pieces3x3::Edge(Edge::Uf),
    ];
    assert_eq!(
        Mask::<Cube3x3>::from_double_iter(pieces1, pieces1),
        Mask::<Cube3x3>::from_double_iter(pieces2, pieces2)
    );
}

#[test]
fn mask_ignores_order() {
    let pieces1 = [
        Pieces3x3::Center(Center::F),
        Pieces3x3::Center(Center::U),
        Pieces3x3::Edge(Edge::Uf),
    ];
    let pieces2 = [
        Pieces3x3::Center(Center::F),
        Pieces3x3::Edge(Edge::Uf),
        Pieces3x3::Center(Center::U),
    ];
    assert_eq!(
        Mask::<Cube3x3>::from_double_iter(pieces1, pieces1),
        Mask::<Cube3x3>::from_double_iter(pieces2, pieces2)
    );
}

#[test]
fn masked_default_applies_to_default_but_not_r() {
    let pieces1 = [
        Pieces3x3::Center(Center::F),
        Pieces3x3::Center(Center::U),
        Pieces3x3::Edge(Edge::Ur),
    ];
    let pieces2 = [
        Pieces3x3::Center(Center::F),
        Pieces3x3::Edge(Edge::Ur),
        Pieces3x3::Center(Center::U),
    ];
    assert!(Mask::from_double_iter(pieces1, pieces2).applies_to(&Cube3x3::default()));
    assert!(
        !Mask::from_double_iter(pieces1, pieces2).applies_to(&Cube3x3::from_solved("R").unwrap())
    );
}

#[test]
fn masked_default_applies_to_default_and_r_if_mask_excludes_r() {
    let pieces1 = [
        Pieces3x3::Center(Center::F),
        Pieces3x3::Center(Center::U),
        Pieces3x3::Edge(Edge::Ul),
    ];
    let pieces2 = [
        Pieces3x3::Center(Center::F),
        Pieces3x3::Edge(Edge::Ul),
        Pieces3x3::Center(Center::U),
    ];
    assert!(Mask::from_double_iter(pieces1, pieces2).applies_to(&Cube3x3::default()));
    assert!(
        Mask::from_double_iter(pieces1, pieces2).applies_to(&Cube3x3::from_solved("R").unwrap())
    );
}

#[test]
fn masked_with_correct_permutation_but_wrong_orientation_parses_correctly() {
    let pieces = [Pieces3x3::Corner(Corner::Ufr)];
    assert!(Mask::from_double_iter(pieces, []).applies_to(&Cube3x3::from_solved("R U").unwrap()));
    assert!(!Mask::from_double_iter([], pieces).applies_to(&Cube3x3::from_solved("R U").unwrap()));
}

#[test]
fn masked_with_correct_orientation_but_wrong_permutation_parses_correctly() {
    let pieces = [Pieces3x3::Corner(Corner::Ufr)];
    assert!(!Mask::from_double_iter(pieces, []).applies_to(&Cube3x3::from_solved("U").unwrap()));
    assert!(Mask::from_double_iter([], pieces).applies_to(&Cube3x3::from_solved("U").unwrap()));
}

#[test]
fn empty_mask_always_applies() {
    for seed in 0..100 {
        assert!(Mask::from_iter([], []).applies_to(&Cube3x3::random_state_with_seed(seed)));
    }
}
