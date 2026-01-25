use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rquickjs::atom::PredefinedAtom;
use rquickjs::class::Trace;
use rquickjs::function::Args;
use rquickjs::{
    AsyncContext, Class, Ctx, FromIteratorJs, Function, IntoJs, Null, Object, Promise, Value,
};
use rquickjs::{JsLifetime, Result};

use crate::ffi::SharedMap;
use crate::handle_js_err;

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
pub async fn run_on_request(context: AsyncContext) {
    context
        .async_with(async |ctx| {
            let global = ctx.globals();
            let on_req: Function = global.get("onRequest").unwrap();

            let mut m = HashMap::new();
            m.insert("from_rust_side".to_string(), "yes".to_string());

            let map = SharedMap::new(Arc::new(Mutex::new(m)));
            let req = Class::instance(ctx.clone(), Request::new_rust(map.clone())).unwrap();
            let mut args = Args::new(ctx.clone(), 1);
            let value = req.into_js(&ctx).unwrap();
            args.push_arg(value).unwrap();

            // call js function with rust struct
            match on_req
                .call_arg::<Promise>(args)
                .unwrap()
                .into_future::<()>()
                .await
            {
                Err(err) => handle_js_err(&ctx, err),
                _ => {}
            }
            // get final map
            assert!(map.lock().get("from_js_side").is_some());
            assert!(map.lock().get("from_rust_side").is_some());
        })
        .await
}
