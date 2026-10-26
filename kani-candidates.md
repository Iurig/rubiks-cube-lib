# Kani candidates

Places in the crate that suit Kani proofs, ranked by ease over utility. Written
2026-10-25 against the `kani` branch. Every snippet here contains `???` on
purpose, so none of them compile.

## Problems in the first harness

### 1. A normal build fails

`cargo check -p rubiks-cube` gives:

```
error[E0433]: cannot find module or crate `kani` in this scope
  --> crates/rubiks-cube/src/zn.rs:13:53
```

The `kani` crate only exists in two cases. `cargo kani` injects it, and the
`[target.'cfg(kani_ra)'.dependencies]` entry adds it, but only when the
`kani_ra` cfg is set. The derive on `Zn` isn't gated, so plain `cargo build` and
`cargo test` can't find the crate. The Kani book gates derives like this:

```rust
#[cfg_attr(kani, derive(kani::Arbitrary))]
```

`cfg_attr(cond, attr)` applies `attr` only when `cond` holds.

### 2. Deriving `Arbitrary` on `Zn` produces invalid values

The derive builds a `Zn` straight from its field, so `kani::any()` can return
any `u8`, such as `Zn::<2>(200)`. That skips `new`, the only thing that keeps
the value below `N`. The current harness still passes, because `2 + 200 - 200`
is fine. But `Neg` would compute `2 - 200`, Kani would report an overflow, and
that failure would be a false alarm about a value the code can never create.

The fix is to implement the trait by hand and build the value through `new`.
`kani::Arbitrary` has one method, `fn any() -> Self`:

```rust
#[cfg(kani)]
impl<const N: usize> kani::Arbitrary for Zn<N> {
    fn any() -> Self {
        ??? // go through the constructor that keeps the invariant
    }
}
```

Run all harnesses with `cargo kani -p rubiks-cube`. Add `--harness <name>` to
run just one.

## Tier 1: small enums and integers, no loops

### 1. `Zn` laws ([zn.rs](crates/rubiks-cube/src/zn.rs))

- For any `usize`, `new(v).value() < N`.
- No operator overflows on valid inputs.
- `x - y == x + (-y)`, `-(-x) == x`, plus associativity and distributivity.
- Pick concrete moduli: 2, 3, and 256.
- What Kani adds over the tests: they loop over `0..5` and spot-check 256. Kani
  covers every `usize` passed to `new` and every pair at `N = 256`.

### 2. Involutions on move enums

- [MoveModifier::inverse](crates/rubiks-cube/src/puzzles/cube3x3/moves.rs#L116):
  `m.inverse().inverse() == m`.
- `Move3x3::inv` twice gives the original move.
- [Face::opposite](crates/rubiks-cube/src/puzzles/cube3x3/moves/table.rs#L116)
  is an involution with no fixed point.
- The only setup is `Arbitrary` on the enums. Put the `cfg_attr` derive above on
  `MoveModifier`, `MovablePart`, `Slice` and `Rotation`, and inside `new_piece!`
  at [pieces.rs:257](crates/rubiks-cube/src/puzzles/cube3x3/pieces.rs#L257) so
  every piece enum gets it. An enum like `MovablePart` can derive it once its
  fields' types implement it.

### 3. `Indexed` round trips, including the crate's only `unsafe`

- Impls: [MovablePart](crates/rubiks-cube/src/puzzles/cube3x3/moves.rs#L126),
  `Move3x3`, [Piece3x3](crates/rubiks-cube/src/puzzles/cube3x3/mod.rs#L96), the
  piece enums, and the pair impl at
  [indexed.rs:225](crates/rubiks-cube/src/indexed.rs#L225), tested with concrete
  types.
- Properties: `x.index() < COUNT`, and `from_index(x.index()) == x`.
  `assert_round_trips` checks the other direction, index to value to index.
- [piece::index](crates/rubiks-cube/src/piece.rs#L34) reads through a raw
  pointer. Kani checks that read for undefined behaviour on every piece, and can
  prove the result equals `piece as usize`. Ordinary tests can't detect
  undefined behaviour, so this is the most useful item in Tier 1.

### 4. `table_index` ([table.rs:45](crates/rubiks-cube/src/puzzles/cube3x3/moves/table.rs#L45))

- It's a pure `const fn` that guards the `ALL_MOVES[...]` index in `cube_state`.
- Properties: the result is in bounds, and two moves get the same index only if
  they have the same part and the same turn class (Double and DoublePrime share
  one).
- It's private, so put the `#[cfg(kani)] mod` inside `table.rs`.

### Caveat

For items 2–4 the inputs are few enough that an exhaustive loop test proves the
same thing. They're good for learning Kani, not for catching what the tests
miss. The exception is the undefined behaviour check in item 3.

## Tier 2: fixed arrays and loops

### 5. Group laws on `PieceConfiguration`

Functions: [then](crates/rubiks-cube/src/piece.rs#L157),
[inv](crates/rubiks-cube/src/piece.rs#L71),
[cycle](crates/rubiks-cube/src/piece.rs#L167).

- Properties: `x.then(&x.inv()) == IDENTITY`, `IDENTITY.then(&x) == x`, and
  associativity.
- This is where Kani beats tests: it covers all 8!·3⁸ corner states, not
  samples.
- It needs two features beyond `kani::any`:
  - **`kani::assume(cond)`** drops every input for which `cond` is false. It's
    needed because `kani::any::<[Corner; 8]>()` allows repeated corners, which
    isn't a permutation:
    ```rust
    let perm: [Corner; 8] = kani::any();
    // for every pair i < j: kani::assume(perm[i] != perm[j]);
    ???
    ```
  - **`#[kani::unwind(n)]`** goes on the harness. Kani can only explore loops up
    to a fixed number of passes. If `n` is too small, it reports an "unwinding
    assertion" failure. For an 8-pass loop, use 9, one more than the loop runs.
- Also needs the hand-written `Zn` `Arbitrary` from above.
- Do corners (8) before edges (12). Associativity uses three symbolic states and
  may be slow.

## Tier 3: possible but expensive

- **[Move3x3::from_str](crates/rubiks-cube/src/puzzles/cube3x3/moves.rs#L216)
  never panics on short input.** Build the input from a `[u8; 3]` and assume
  `from_utf8` succeeds. The error paths call `to_string`, which allocates and
  slows Kani. A parse/`Display` round trip also goes through formatting, which
  is very slow under Kani.
- **[parity](crates/rubiks-cube/src/piece.rs#L121).** It uses a `Vec`,
  `contains`, and nested loops. Kani handles that only at small sizes.

## Skip

- **The solver and search.** They use a `HashMap` memo, `Arc` and rayon, and
  Kani doesn't support concurrency.
- **The `LazyLock` move table.** The tests already cover all of its moves.
- **`FxHasher`.** It has no property worth proving.
- **`Pow::pow`.** It recurses on a symbolic exponent, and the types that
  implement it are big.
- **Scrambling with `fastrand`, and `facelets.rs`.**

## Sources

- [Kani tutorial: first steps](https://model-checking.github.io/kani/tutorial-first-steps.html)
- [Kani: Rust feature support](https://model-checking.github.io/kani/rust-feature-support.html)
- [Kani tutorial: loop unwinding](https://model-checking.github.io/kani/tutorial-loop-unwinding.html)
- [Kani tutorial: nondeterministic variables](https://model-checking.github.io/kani/tutorial-nondeterministic-variables.html)
- [Kani: A Model Checker for Rust (summary)](https://www.emergentmind.com/videos/kani-a-model-checker-for-rust-2f2ad5d8)
- [kani-verifier skill guide](https://openskillindex.com/skills/majiayu000-claude-skill-registry-kani-verifier)
