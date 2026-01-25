use crate::{ffi, handle_js_err};
use rquickjs::{AsyncContext, Function, Result, Undefined};
use rquickjs::{Ctx, IntoAtom, IntoJs, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
fn register_ffi(ctx: &Ctx<'_>) {
    let global = ctx.globals();
    global
        .set(
            "__ffi_print",
            Function::new(ctx.clone(), ffi::ffi_print)
                .unwrap()
                .with_name("__ffi_print")
                .unwrap(),
        )
        .unwrap();
}
pub async fn setup_runtime_context(context: &AsyncContext, code: &str) {
    context
        .async_with(async |ctx| {
            register_ffi(&ctx);
            match ctx.eval::<Undefined, _>(code) {
                Err(err) => handle_js_err(&ctx, err),
                _ => {}
            }
        })
        .await;
}

fn ffi_print(s: String) {
    println!("{s}");
}
#[derive(Default, Clone, Debug)]
pub struct SharedMap<T, S>(Arc<Mutex<HashMap<T, S>>>);
impl<T, S> SharedMap<T, S> {
    pub fn new(m: Arc<Mutex<HashMap<T, S>>>) -> Self {
        Self(m)
    }
    pub(crate) fn lock(&self) -> MutexGuard<'_, HashMap<T, S>> {
        self.0.lock().unwrap()
    }
}
impl<'js, K, V> IntoJs<'js> for SharedMap<K, V>
where
    K: IntoAtom<'js>,
    V: IntoJs<'js>,
{
    fn into_js(self, _ctx: &Ctx<'js>) -> Result<Value<'js>> {
        unimplemented!(
            "On javascript side, SharedMap should only be modified by function, not access directly."
        );
    }
}
