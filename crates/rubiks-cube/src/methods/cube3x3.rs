use std::sync::LazyLock;

use crate::{Cube3x3, methods::search_step::MemoCache};

#[macro_use]
mod helpers;
pub mod kociemba;
pub mod roux;
pub mod zz;

/// The search memos of every [`Cube3x3`] method, shared for the whole program. Two steps that
/// bring the same pieces home with the same sequences grow one memo, even when different
/// methods, or different options of one method, build them.
static MEMOS: LazyLock<MemoCache<Cube3x3>> = LazyLock::new(MemoCache::new);
