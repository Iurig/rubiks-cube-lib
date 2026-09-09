# rubiks-cube-lib

![CI](https://github.com/Iurig/rubiks/actions/workflows/ci.yml/badge.svg)
![License](https://img.shields.io/github/license/Iurig/rubiks)


A Rust library that models the 3×3×3 Rubik's Cube, with planned expansion for other cube types, as well as solving from an algorithm library.

A cube state is a value of type `Cube3By3`. Moves are cube states too, and applying a move is just
group multiplication. Everything is `Copy`, and most operations are `const fn`,
so the entire move table is built at compile time.

> **Status: early work in progress.** The core group structure and all non-wide moves work and are well tested. Wide moves are not implemented yet, and integration
> tests that use them fail as expected. See [Current state](#current-state).

## Quick example

```rust
use rubiks_cube_lib::{Cube3By3, Inv, Pow};

// Apply a sequence to the solved cube, example is Sebastiano Tronto's 16 move FMC WR
let scramble = Cube3By3::from_solved("R' U' F D2 L2 F R2 U2 R2 B D2 L B2 D' B2 L' R' B D2 B U2 L U2 R' U' F");
let solution = Cube3By3::from_solved("D2 F' D2 U2 F' L2 D R2 D B2 F L2 R' F' D U'");

// Composition is multiplication: left operand first, then right
assert!((scramble * solution).is_solved());

// Inverses and powers
assert_eq!(Cube3By3::from_solved("R").inverse(), Cube3By3::from_solved("R'"));
assert!(Cube3By3::from_solved("R U R' U'").pow(6).is_solved());

// Chain moves onto an existing state
let cube = Cube3By3::IDENTITY.move_sequence("R U").move_sequence("R' U'");
assert_eq!(cube, Cube3By3::from_solved("R U R' U'"));
```

## Notation

Move strings use standard cube notation. Whitespace and newlines separate moves, and anything
after `//` on a line is a comment, so you can paste annotated reconstructions directly:

```rust
let solved = Cube3By3::from_solved("
    U' L2 D' B2 D R2 F2 D' B2 R2 D B' R F2 R D' B' F U2 R' U D   // scramble
    y2 F' M F' R U' R U' Fw z'                                  // FB
    U R U r M' U' R U2' R'                                      // SS
    ...
");
```

| Kind        | Tokens                       | Implemented           |
| ----------- | ---------------------------- | --------------------- |
| Face turns  | `R` `L` `U` `D` `F` `B`      | Yes                   |
| Slice moves | `M` `E` `S`                  | Yes    |
| Rotations   | `x` `y` `z`                  | Yes|
| Wide moves  | `Rw` … or lowercase `r` …    | Not yet               |

All these work with the following 4 modifiers: ` `(none), `'`, `2` and `2'`

Lowercase face letters (`r`, `u`, `f`, `b`, `l`) are rewritten to `Rw`, `Uw`, ... before lookup.

## How it works

The cube is the direct product of three independent piece groups:

- **Centers**: 6 pieces, no orientation. Only slice moves and rotations move them. The other pieces are expressed as if these were not moved (i.e. moves that move them will be translated to face turns).
- **Corners**: 8 pieces, orientation in ℤ/3ℤ.
- **Edges**: 12 pieces, orientation in ℤ/2ℤ.

Each is a `PieceConfiguration<P, N, O>`: a permutation array of `N` pieces of type `P` plus an
orientation array of `N` values in `ZnRing<O>`. Composition (`then`) permutes and adds orientations;
inversion negates them. `Cube3By3` just composes its three configurations component-wise.

Piece types are fieldless `#[repr(u8)]` enums implementing the `SinglePiece` trait, which lets the
library index arrays by piece at zero cost. Ordering conventions follow blindfolded-solving
standard memorization order, except for edges which are clockwise by layer, starting on the last and going to the first:

- Centers: `U F R B L D`
- Corners: `UBL UBR UFR UFL DFL DFR DBR DBL`
- Edges: `UB UR UF UL FL FR BR BL DF DR DB DL`

Corner orientation counts clockwise twists relative to the U/D sticker being in the U or D layer; edge orientation is 0 for
oriented, 1 for flipped.

Every clockwise move is written down once as an explicit cycle in [src/cube3by3/moves.rs](src/cube3by3/moves/table.rs).
Its inverse and double are derived at compile time into a single `ALL_MOVES` table that string
parsing looks up.

## Public API

| Item                                     | Purpose                                                    |
| ---------------------------------------- | ---------------------------------------------------------- |
| `Cube3By3`                               | The cube state. `Default` and `IDENTITY` are the solved cube. |
| `Cube3By3::from_solved(&str)`            | Apply a move string to the solved cube, returning the moved cube.                    |
| `Cube3By3::move_sequence(&self, &str)`   | Apply a move string to this state, returning a new one.    |
| `Cube3By3::is_solved()`                  | Equality with the identity. Ignores a rotation implementation that only moves centers. |
| `Cube3By3::respects_orientation_parity()`| Corner twists sum to 0 mod 3 and edge flips to 0 mod 2. Permutation parity check not yet implemented.    |
| `impl Mul for Cube3By3`                  | `a * b` applies `a` then `b`. Associative, not commutative. |
| `Inv` trait                              | `inverse()`, implemented for cubes and piece configurations.|
| `Pow` trait                              | `pow(n)`, repeated multiplication.                         |
| `PieceConfiguration`, `SinglePiece`, `index`, `try_from_index` | Generic building blocks for other puzzles. |
| `zn::ZnRing<N>`                          | Integers mod `N`, `const`-friendly, with `Add` and `Neg`.  |

## Project layout

```
src/
  lib.rs                  public exports
  ops.rs                  Inv and Pow traits
  zn.rs                   ZnRing<N>
  single_piece.rs         SinglePiece trait and PieceConfiguration
  string_processing.rs    tokenising move strings, comment stripping, wide-move rewriting
  cube3by3/
    mod.rs                Cube3By3, Mul/Inv/Pow impls, rotation-aware is_solved, orientation parity
    pieces.rs             piece enums, counts, and type aliases for the 3×3
    moves.rs              Move type, MovablePart/MoveModifier enums, string parsing
    moves/
      table.rs            compile-time ALL_MOVES table: 9 hand-written face and slice moves,
                          rotations derived from them, then inverses and doubles of everything
tests/
  testing.rs              integration tests: group laws, move orders, real solve reconstructions
```

## Building and testing

Requires a Rust toolchain with the 2024 edition (Rust 1.85 or newer). No runtime dependencies;
`fastrand` is used only in tests.

```sh
cargo build
cargo test
```

## Current state

All unit tests and doc tests pass. In the integration suite, 18 of 21 tests pass and the
remaining 3 are `#[ignore]`d rather than failing:

- `roux_solve_with_comments`, `roux_solve_without_comments`, `roux_solve_removes_comments`:
  the reconstructions use wide moves (`Fw`, `r`, ...), which have no entry in the move table
  yet, so parsing would panic.

What works today:

- All six face turns, the three slice moves `M`, `E`, `S`, and the three rotations `x`, `y`, `z`,
  each with the `'`, `2`, and `2'` modifiers. The rotations are derived at compile time from
  the face and slice moves, so they cannot drift out of sync with them.
- `is_solved` is rotation-aware: it re-orients the cube by its centers before comparing with
  the identity, so `Cube3By3::from_solved("x y2 z'")` reports solved.
- `respects_orientation_parity` checks corner twist and edge flip sums. There is no
  permutation parity check yet.

Wide moves are parsed (lowercase `r` is rewritten to `Rw`, and the `w` suffix is recognised)
but have no move-table entry, so using one panics with a message naming the move.

## Roadmap

- Implement wide moves (`Rw`, `Uw`, ...) by composing a face turn with the parallel slice or
  rotation, then un-ignore the three Roux integration tests.
- Add a permutation parity check alongside `respects_orientation_parity`, so a full
  "is this a reachable state" predicate can be exposed.
- Return a `Result` from move-string parsing instead of panicking on unknown tokens.
- Solving from an algorithm library, as mentioned in the introduction.
- Reuse `PieceConfiguration` / `SinglePiece` for other puzzles; the string-processing module was
  split out with that in mind.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for the full text.
