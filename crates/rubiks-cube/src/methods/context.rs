use std::{
    any::{Any, TypeId},
    sync::{Arc, LazyLock, Mutex},
};

use crate::fast_hash::FxMap;

pub type UntypedCache = Arc<dyn Any + Send + Sync>;

#[derive(Default, Debug)]
pub struct BuildContext {
    caches: Mutex<FxMap<TypeId, UntypedCache>>,
}

impl BuildContext {
    pub fn get_or_create<C: Any + Default + Send + Sync>(&self) -> Arc<C> {
        let mut map = self
            .caches
            .lock()
            .expect("the mutex should not be poisoned");
        map.entry(TypeId::of::<C>())
            .or_insert_with(|| Arc::new(C::default()))
            .clone()
            .downcast::<C>()
            .expect("entry keyed by TypeId::of::<C>() always holds a C")
    }
}

/// The memos of every [`Method`](crate::Method) of every [`Puzzle`](crate::Puzzle), shared for the
/// whole program. Two steps of the same type that bring the same pieces home with the same
/// sequences grow one memo, even when different methods, or different options of one method, build
/// them.
pub(crate) static GLOBAL: LazyLock<BuildContext> = LazyLock::new(Default::default);
