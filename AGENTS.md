# AGENTS.md

Guidance for Codex in this repository.

## Working agreement

The owner writes all code in `src/` and `tests/` themselves; this is a Rust learning project, and the user is a beginner at both programing and the Rust language. Review, explain, design, and write docs. Prefer explaining to showing final code; put proposed code in chat as snippets when it helps, and edit `src/` or `tests/` only when explicitly asked in that conversation. Never commit unless explictly asked.

**When grilling, ask exactly one question per message, then stop and wait.** This overrides the grilling skill's "ask the whole frontier in one round" instruction. Do not number questions, do not batch a frontier, do not add a second question "since it is independent". Pick the frontier question whose answer unblocks the most, give the recommended answer, and end the turn. Facts you can look up yourself still get looked up before asking.

`AGENTS.md`, `CONTEXT.md`, and `docs/adr/` are excluded from git through `.git/info/exclude` and exist for communication between the owner and Codex. Edit them freely, without asking: keep them current as the code changes, record decisions and vocabulary as they crystallise, and prune what stops being true. The owner rarely edits them and they are not part of what the owner is learning. Never stage them or link to them from tracked files.

Vocabulary lives in `CONTEXT.md`; use its terms. The owner says "move", never "token", for one unit of notation text; a bad one is an "invalid move" (cube state, move, part, modifier, move table, base move, derived move, handedness, slot, piece, twist, flip, reachable). Decisions live in `docs/adr/`; read them before touching `moves.rs`, `moves/table.rs`, or the `Move` type.

## Commands

```sh
cargo test                                   # unit + integration + doc tests
cargo test r_prime_is_inverse_of_r           # one test by name (substring match)
cargo test --test testing                    # integration suite only
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings    # CI fails on any warning
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

CI runs those on Linux and Windows with `RUSTFLAGS=-D warnings`. `Cargo.toml` turns on `pedantic`, `nursery`, and restriction lints (`unwrap_used`, `indexing_slicing`, `panic`, ...); `clippy.toml` relaxes them inside tests and `const` contexts, so index freely in a `const fn` and prefer `.get()` in runtime code.

## Architecture

The cube is a group. `Cube3By3` is the direct product of three `PieceConfiguration<P, N, O>` values (centers, corners, edges): a permutation array of `N` pieces plus an orientation array in `Zn<O>`. Moves are cube states too, and `a * b` applies `a` **then** `b`, left to right like notation. The subtle line is composition in `piece.rs`: the new orientation at slot `i` is `self.orientation[index(other.permutation[i])] + other.orientation[i]`.

Pieces are fieldless `#[repr(u8)]` enums; `index(piece)` reads the discriminant, and a `const _` block in `cube3by3/pieces.rs` asserts `ALL[i] as usize == i`. Ordering conventions are in the comments beside those enums.

**The move table is `const`.** `cube3by3/moves/table.rs` builds the whole table at compile time: nine hand-written base moves, then rotations, wide moves, inverses, and doubles derived from them (ADR-0002). Stay const-compatible there: `while` loops, the `const_mul`/`const_inverse`/`const_add`/`const_neg` twins instead of trait methods, no `format!`. Slot order comes from the enums: `table_index_clockwise` is group offset plus discriminant, and a `const _` block asserts every `ALL_MOVES` entry sits at its own index. The base-move literal is read only by the placement loop, which stores each entry by `part`; derivations read the placed array, so the literal's order is free.

**Handedness is the bug class here.** A slice or rotation entry can be its own inverse and still pass every order, period, and involution test, because those are invariant under inverting a generator; the E-slice bug (commit 54ac1ee) survived the suite that way. Tests that pin handedness are facts from the physical cube (where one named piece ends up, written through `piece_at` in `tests/testing.rs`) and real reconstructions in the same file; `table.rs` tests pin only the derivations. Derivation identities like `y == U E' D'` pin only the derivation (ADR-0002).

**One notation path.** `Cube3By3::move_sequence` → `Move::sequence` (comment strip, whitespace split) → `TryFrom<&str> for Move` (one move; a lowercase face letter is the wide move) → table lookup. Errors are `ParseSequenceError` (line and position, 1-based, wrapping `ParseMoveError`), both exported from `lib.rs`; tests and the README use `Box<dyn Error>`. `lib.rs` includes `README.md` as the crate docs, so its `rust` blocks are doctests and non-Rust blocks need a `text` tag. A lone move and a multi-line reconstruction take the same path, and a string with no moves is a no-op; `empty_and_comment_only_sequences_have_no_moves`, `lowercase_face_is_the_wide_move`, and `sequences_without_moves_leave_the_cube_unchanged` pin this. Only `Cube3By3` and a few traits are exported, so integration tests reach moves through strings; tests that need `Move` or table internals live in the inline `#[cfg(test)]` module of the file they test.

**Cube state queries.** `Cube3By3` exposes `corners()`, `edges()`, `centers()` by reference; `PieceConfiguration` answers `piece_at(slot)` and `orientation_at(slot)`. Slot and piece share an enum (ADR-0004), so `piece_at(Ubr) == Ufr` reads "the UFR piece sits in the UBR slot". `then`, `cycle`, `const_inverse`, `parity`, and `orientation_sum` are `pub(crate)`; the public type is read-only. `Piece` is sealed through `piece::private::Sealed`, implemented by the `new_piece!` macro; the module is `pub` only because `redundant_pub_crate` fires on `pub(crate)` inside the private `piece` module, and it is unreachable from outside anyway.

`is_solved` re-orients by named center slots with at most one `y`, four `x`, and four `z` (`rotated_until_solved_centers`, which returns `None` when the centers are not a whole-cube rotation), then compares with `IDENTITY`. `is_reachable` checks twist sum, flip sum, the three-way permutation parity (a `Zn<2>` per configuration, by cycle decomposition), and that the centers form a rotation.

## Direction

Stated by the owner on 2026-09-10; not a spec, and not yet grilled. The crate grows into a **human-like solver CLI**: it solves a scramble the way a person would, with CFOP, Roux, ZZ, or Petrus. A method is a sequence of steps, and every step is one of two kinds: a **searched step**, found by search within a restricted moveset (the owner has heard of Kociemba's two-phase algorithm and is open to borrowing from it), or an **algorithm step**, applied from a library of human algorithms (OLL, PLL, CMLL, COLL, ZBLL). Output is a reconstruction in the crate's notation, which is why printing moves back and the distinct `2'` (ADR-0003) matter. The known-gap chain (public `Move`, typed error, part module) is groundwork for this. The seeded wayfinder map is `.scratch/solver/map.md`; grilling or design sessions that touch anything solver-shaped should read it and keep decisions there. Terms already pinned: `CONTEXT.md` under Solving.

## Known gaps

Each names what closes it.

- The 18 parts are hand-listed in `Display for MovablePart` and in `TryFrom<&str>`; `MovablePart::ALL` (test-only, `moves.rs`) is the candidate single source. Closed when both derive from one list.
- `IMPLEMENTED_MOVES` in `tests/testing.rs` has 12 entries. Closed when the order, inverse, and involution tests iterate all 18 parts.
- `rotated_until_solved_centers` names the same piece as `Center::F` and `Faces::F` in one function. Cosmetic; closed by issue 04, which owns the `Faces` alias.
- `Move` is private and the owner wants it public. Blocked on the single part list (issue 04); the typed error landed in 6965e8d. Do not export it before 04.

## Agent skills

### Unslop

Apply `/unslop` to every message to the owner and to every human-facing document: `.learn/`, `README.md`, doc comments, and commit messages proposed in chat.

Documents that both the owner and agents read (`AGENTS.md`, `CONTEXT.md`, `docs/adr/`, `docs/agents/`, `.scratch/`) are written for agents first, with the `writing-for-agents` skill: facts before rationale, repo paths and commit hashes instead of descriptions, one meaning in one place, no persuasion. Unslop's plain-speech rules (27 to 33) apply to them too, because an agent reads whole sentences with less decoding than fragments. 

### Issue tracker

Issues and specs are local markdown under `.scratch/<feature>/`, untracked like the other agent docs. See `docs/agents/issue-tracker.md`.

### Triage labels

The five default roles, as `Status:` lines in issue files. Implementation tickets are `ready-for-human`, never `ready-for-agent`, because the owner writes the code. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: `CONTEXT.md` and `docs/adr/` at the root. See `docs/agents/domain.md`.

## Learning workspace

The owner is learning Rust and production practice through this repo. The teaching workspace is `.learn/`, excluded from git like the other agent docs, with a local pre-commit hook (`.git/hooks/pre-commit`) that refuses commits staging any of them. `/teach`, `/learn`, "next lesson", or "what should I learn next" all mean: follow the `teach` skill (`~/.Codex/skills/teach`) with `.learn/` as its workspace root. Read `.learn/MISSION.md`, `NOTES.md`, `PLAN.md`, and `learning-records/` before picking anything. Lessons cite repo paths and use the open tickets in `.scratch/` as practice; the owner writes all practice code and makes all commits. Ground claims in `.learn/RESOURCES.md`, not memory. Learning record 0001 says what Exercism already proves fluent; do not re-teach it.
