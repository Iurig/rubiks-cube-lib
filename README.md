# rubiks-cube-lib

![CI](https://github.com/Iurig/rubiks/actions/workflows/ci.yml/badge.svg)
![License](https://img.shields.io/github/license/Iurig/rubiks)
![MSRV](https://img.shields.io/badge/rustc-1.85%2B-blue)


A Rust library that models the 3×3×3 Rubik's Cube, with planned expansion for other cube types, as well as solving from an algorithm library.

A cube state is a value of type `Cube3By3`. Moves are cube states too, and applying a move is just
group multiplication. Everything is `Copy`, allocation-free, and most operations are `const fn`,
so the entire move table is built at compile time.

> **Status: early work in progress.** The core group structure and all six face turns work and are well tested. The slice moves `M`, `E`, `S` are defined but currently
> **incorrect**. The rotations and wide moves are not implemented yet, and integration
> tests that use them fail as expected. See [Current state](#current-state).

## Quick example

```rust
use rubiks::{Cube3By3, Inv, Pow};

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
| Slice moves | `M` `E` `S`                  | Defined, but wrong    |
| Rotations   | `x` `y` `z`                  | `y` defined, all wrong|
| Wide moves  | `Rw` … or lowercase `r` …    | Not yet               |

All these work with the following 4 modifiers: ` `(none), `'`, `2` and `2'`

Lowercase face letters (`r`, `u`, `f`, `b`, `l`) are rewritten to `Rw`, `Uw`, ... before lookup.
`x`, `z`, and wide moves are recognised by the parser but have no entry in the move table yet, so
using them panics with a message naming the move. The slice moves do have entries, but their
definitions are known to be wrong. Do not rely on `M`, `E`, or `S` results until this is fixed.

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

Every clockwise move is written down once as an explicit cycle in [src/cube3by3/moves.rs](src/cube3by3/moves.rs).
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
| `PieceConfiguration`, `SinglePiece`, `index`, `from_index` | Generic building blocks for other puzzles. |
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
    mod.rs                Cube3By3, Mul/Inv/Pow impls, is_solved, parity
    pieces.rs             piece enums, counts, and type aliases for the 3×3
    moves.rs              Move type, string parsing, compile-time move table
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

Unit tests all pass. In the integration suite, 14 of 19 tests pass. The 5 failures are known and
document unfinished work rather than regressions:

- `cfop_solve`, `roux_solve_*` (4 tests): use `x`, `z`, `Fw`, or `r`, which are not implemented
  yet, so parsing panics.
- `slice_face_has_constant_and_correct_period`: exposes the incorrect slice-move definitions.
  `M U` reports solved after 4 repetitions rather than the expected 12.

One unit test, `rotated_solved_is_solved`, is `#[ignore]`d pending rotation-aware `is_solved`.

## Roadmap

- Fix the `M`, `E`, and `S` slice-move definitions.
- Implement `x` and `z` rotations and wide moves (`Rw`, `Uw`, ...).
- Make `is_solved` ignore whole-cube rotation, i.e. treat `Cube3By3::from_solved("y")` as solved.
- Reuse `PieceConfiguration` / `SinglePiece` for other puzzles; the string-processing module was
  split out with that in mind.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for the full text.
