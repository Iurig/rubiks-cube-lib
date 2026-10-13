#![doc = include_str!("DESIGN.md")]
#![allow(
    clippy::pedantic,
    clippy::nursery,
    clippy::allow_attributes_without_reason,
    missing_docs
)]

use rubiks_cube::{cube3x3_method_helpers::*, *};

fn main() {
    let eolr = SearchStep::builder("EOLR", PieceSet::from_algset(&algs!("M\n U2")))
        .search_algs(parts!("M U"));

    let advanced_roux = Technique::<Cube3x3>::default()
        .add("FB", ["R", "U", "F", "B", "D", "M", "r"]) // Sets a step with no goal and the given moves.
        .add("SB", ["R", "U", "r", "M"]) // Previous step goal: whatever isn't touched by the moves.
        .add_with_goal(
            // explicit goal defining
            "CMLL",
            [include_str!("path/to/algs/"), "U"], /* Algs to be used, isolated quarter
                                                   * turns are interpreted as multiple
                                                   * one move algorithms, given by all
                                                   * their variants. This still sets
                                                   * previous step goal to whatever
                                                   * isn't touched by these. */
            algs!(["U M' U'", "M"]), /* The goal: these algs don't move corners, so they enforce
                                      * corners to be solved. */
        )
        .add_step(eolr) // explicitly defined step, doesn't touch the previous step goal because it is defined
        .finish("LSE", ["M", "U"]); // goal: to solve the cube

    let advanced_cfop = Technique::<Cube3x3>::with_neutrality(["x2", "y"]) // white or yellow bottom
        .add("Cross", ["R", "F", "U", "L", "D", "u"])
        .add_until_all(
            "F2L", // Solves many steps, always choosing the shortest to solve at every point.
            ["R U* R'", "L U* L'", "R' U* R", "L' U* L", "y"], /* The moveset should be the same
                    * for all steps. */
            [["R U* R'"], ["L U* L'"], ["R' U* R"], ["L' U* L"]], /* The goals of each step are
                                                                   * defined by as usual: by the
                                                                   * pieces not touched by each
                                                                   * set of algs. */
        )
        .add_with_goal("OLL", [include_str!("path/to/algs/"), "U"], parts!("U"))
        .finish("PLL", [include_str!("path/to/algs")]);
}
