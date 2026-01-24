use rquickjs::Null;
use rquickjs::atom::PredefinedAtom;
use rquickjs::class::Trace;
use rquickjs::{Ctx, FromIteratorJs, IntoAtom, IntoJs, Object, Value};
use rquickjs::{JsLifetime, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Trace, JsLifetime, Debug)]
#[rquickjs::class]
pub struct Request {
    #[qjs(skip_trace)]
    // on javascript, js code can only access field by function
    // so we can modify value directly on rust side
    headers: SharedMap<String, String>,
}
impl Request {}

#[rquickjs::methods]
impl Request {
    #[qjs(constructor)]
    pub fn new() -> Self {
        Self {
            headers: Default::default(),
        }
    }
    #[qjs(skip)]
    pub fn new_rust(map: SharedMap<String, String>) -> Self {
        Self { headers: map }
    }
    #[qjs(rename = "setHeader")]
    fn set_header(&mut self, k: String, v: String) {
        self.headers.lock().insert(k, v);
    }
    #[qjs(rename = "data")]
    fn data(&self) -> SharedMap<String, String> {
        self.headers.clone()
    }
    #[qjs(rename = PredefinedAtom::ToJSON)]
    fn to_json<'js>(&self, ctx: Ctx<'js>) -> Result<Value<'js>> {
        let obj = Object::new(ctx)?;
        obj.set("data", &*self.headers.lock())?;
        Ok(obj.into_value())
    }

    #[allow(clippy::inherent_to_string)]
    #[qjs(rename = PredefinedAtom::ToString)]
    fn to_string(&self) -> String {
        format!("Request( header: {:?})", self.headers)
    }

    #[qjs(rename = "Symbol.toPrimitive")]
    fn to_primitive<'js>(&self, ctx: Ctx<'js>, hint: String) -> Result<Value<'js>> {
        if hint == "string" {
            let s = format!("{:?}", &self.headers);
            return Ok(rquickjs::String::from_str(ctx, &s)?.into_value());
        }
        if hint == "Object" {
            return Ok(rquickjs::Object::from_iter_js(&ctx, &*self.headers.lock())?.into_value());
        }
        Ok(Null.into_value(ctx))
    }
}
#[derive(Default, Clone, Debug)]
pub struct SharedMap<T, S>(pub(crate) Arc<Mutex<HashMap<T, S>>>);
impl<T, S> SharedMap<T, S> {
    fn lock(&self) -> MutexGuard<'_, HashMap<T, S>> {
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
