use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, Mutex, RwLock};

#[derive(Clone)]
struct Callback1(Arc<Mutex<dyn FnMut(&mut dyn Any) + Send + Sync + 'static>>);

impl Debug for Callback1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Callback1()")
    }
}

impl Callback1 {
    fn new(f: Box<dyn FnMut(&mut dyn Any) + Send + Sync>) -> Self {
        Callback1(Arc::new(Mutex::new(f)))
    }

    fn run(&mut self, var: &mut dyn Any) {
        let callback = &mut self.0.lock().unwrap();
        callback(var);
    }
}

struct MapEntry {
    type_id: TypeId,
    cb: Callback1,
}

impl MapEntry {
    fn new(type_id: TypeId, f: Box<dyn FnMut(&mut dyn Any) + Send + Sync>) -> MapEntry {
        MapEntry {
            type_id: type_id,
            cb: Callback1::new(f),
        }
    }
}

lazy_static::lazy_static! {
    static ref REGISTRY: RwLock<HashMap<String, MapEntry>>= Default::default();
}

/// Set the callback for a test value adjustment.
///
/// Usage:
///
/// ```rust
/// use testvalue::{adjust, ScopedCallback};
///
/// fn production_code() {
/// 	let mut var = 1;
/// 	adjust!("adjust_this_var", &mut var);
/// }
///
/// fn test_code() {
///     let _raii = ScopedCallback::new("adjust_this_var", |var| {
/// 	    *var = 2;
///     });
/// }
/// ```
///
pub fn set_callback<S, T, F>(name: S, mut f: F)
where
    S: Into<String>,
    T: Any,
    F: FnMut(&mut T) + Send + Sync + 'static,
{
    let mut registry = REGISTRY.write().unwrap();
    registry.insert(
        name.into(),
        MapEntry::new(
            TypeId::of::<T>(),
            Box::new(move |var| {
                if let Some(var) = var.downcast_mut::<T>() {
                    f(var);
                } else {
                    panic!("Type mismtach");
                }
            }),
        ),
    );
}

/// Set a scoped callback using RAII
#[derive(Debug)]
pub struct ScopedCallback {
    name: String,
}

impl ScopedCallback {
    /// Creates a RAII instance.
    pub fn new<S, T, F>(name: S, f: F) -> Self
    where
        S: Into<String> + Copy,
        T: Any,
        F: FnMut(&mut T) + Send + Sync + 'static,
    {
        set_callback(name.clone(), f);
        ScopedCallback { name: name.into() }
    }
}

impl Drop for ScopedCallback {
    fn drop(&mut self) {
        let mut registry = REGISTRY.write().unwrap();
        registry.remove(&self.name);
    }
}

pub fn internal_adjust<S, T>(name: S, var: &mut T)
where
    S: Into<String>,
    T: Clone + 'static,
{
    let mut registry = REGISTRY.write().unwrap();
    // Clone the var here, since the argument is required to be 'static.
    let mut clone = var.clone();
    if let Some(entry) = registry.get_mut(&name.into()) {
        if (*entry).type_id != TypeId::of::<T>() {
            panic!("Type mismatch");
        }
        (*entry).cb.run(&mut clone);
    }
    *var = clone;
}

/// Define a test value adjustment (requires `testvalue-hook` feature).
#[macro_export]
#[cfg(feature = "testvalue-hook")]
macro_rules! adjust {
    ($name:expr, $var:expr) => {{
        $crate::internal_adjust($name, $var);
    }};
}

/// Define a test value adjustment (disabled, see `testvalue-hook` feature).
#[macro_export]
#[cfg(not(feature = "testvalue-hook"))]
macro_rules! adjust {
    ($name:expr, $var:expr) => {{}};
}
