#![doc = include_str!("DESIGN.md")]
#![allow(
    clippy::pedantic,
    clippy::nursery,
    clippy::allow_attributes_without_reason,
    missing_docs
)]

use rubiks_cube::{cube3x3_method_helpers::*, *};

fn main() {
    let eolr = SearchStep::builder("EOLR", AlgSet::from_static_parts("M U"))
        .goal(PieceSet::from_algset(&AlgSet::from_static("M\n U2")));

    let advanced_roux = Technique::<Cube3x3>::start_with(SearchStep::builder(
        "FB",
        AlgSet::from_static_parts("R U F B D M r"),
    )) // Sets a step with no goal and the given moves.
    .then(SearchStep::builder(
        "SB",
        AlgSet::from_static_parts("R U r M"),
    )) // Previous step goal: whatever isn't touched by the moves.
    .search_with_goal(
        // explicit goal defining
        "CMLL",
        [include_str!("path/to/algs/"), "U"], /* Algs to be used, isolated quarter
                                               * turns are interpreted as multiple
                                               * one move algorithms, given by all
                                               * their variants. This still sets
                                               * previous step goal to whatever
                                               * isn't touched by these. */
        AlgSet::from_static("U M' U' \n M"), /* The goal: these algs don't move corners,
                                              * so they enforce
                                              * corners to be solved. */
    )
    .then(eolr) // explicitly defined step, doesn't touch the previous step goal because it is defined
    .finish("LSE", ["M", "U"]); // goal: to solve the cube

    let advanced_cfop = Technique::<Cube3x3>::builder()
        .with_neutrality(["x2", "y"]) // white or yellow bottom
        .search("Cross", ["R", "F", "U", "L", "D", "u"])
        .until_all(
            "F2L", // Solves many steps, always choosing the shortest to solve at every point.
            ["R U* R'", "L U* L'", "R' U* R", "L' U* L", "y"], /* The moveset should be the same
                    * for all steps. */
            [["R U* R'"], ["L U* L'"], ["R' U* R"], ["L' U* L"]], /* The goals of each step are
                                                                   * defined by as usual: by the
                                                                   * pieces not touched by each
                                                                   * set of algs. */
        )
        .search_with_goall(
            "OLL",
            [include_str!("path/to/algs/"), "U"],
            AlgSet::from_static_parts("U"),
        )
        .finish("PLL", [include_str!("path/to/algs")]);
}
