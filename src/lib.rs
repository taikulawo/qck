use std::{
    any::type_name,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use rquickjs::{
    AsyncContext, AsyncRuntime, Class, Ctx, Error, Function, IntoJs, Module, Promise, Value,
    async_with,
    function::Args,
    loader::{BuiltinResolver, ModuleLoader},
};
use rquickjs_extra::os::OsModule;
use serde::de::DeserializeOwned;

use crate::{ffi::SharedMap, req::Request};

pub mod ffi;
pub mod req;
#[derive(Clone)]
pub struct JsEngine {
    rt: AsyncRuntime,
}
#[derive(Debug)]
pub struct ErrorMessage {
    pub inner: Error,
    pub message: Option<String>,
}
impl From<Error> for ErrorMessage {
    fn from(value: Error) -> Self {
        ErrorMessage {
            inner: value,
            message: None,
        }
    }
}
macro_rules! try_handle_error {
    ($context:ident, $expr:expr) => {
        match $expr {
            Ok(v) => Ok(v),
            Err(err) => Err(::rquickjs::async_with!(&$context => |ctx| {handle_js_err(&ctx,err)}).await)
        }
    };
}
impl JsEngine {
    pub async fn new() -> Result<Self, Error> {
        let rt = AsyncRuntime::new()?;
        let resolver = BuiltinResolver::default().with_module("os");
        let loader = (ModuleLoader::default().with_module("os", OsModule),);
        rt.set_loader(resolver, loader).await;
        let me = Self { rt: rt };
        Ok(me)
    }
    async fn register_global(&self, context: &AsyncContext) {
        async_with!(context => |ctx| {
            ffi::register_ffi(&ctx);
        })
        .await;
    }
    pub async fn run_on_request<T: DeserializeOwned + 'static>(
        &self,
        context: &AsyncContext,
    ) -> Result<T, ErrorMessage> {
        self.run_in_context(context, async |ctx| {
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
            let v = self.call_code_args(&on_req, args).await?;
            // get final map
            assert!(map.lock().get("from_js_side").is_some());
            assert!(map.lock().get("from_rust_side").is_some());
            Ok(v)
        })
        .await
    }

    pub async fn run_in_context<'js, T: DeserializeOwned + 'static>(
        &self,
        context: &AsyncContext,
        callback: impl AsyncFn(Ctx<'_>) -> Result<T, Error>,
    ) -> Result<T, ErrorMessage> {
        let r = async_with!(context => |ctx| {
                callback(ctx).await
        })
        .await;
        try_handle_error!(context, r)
    }
    pub async fn new_context(&self) -> AsyncContext {
        // JSContext represents a Javascript context (or Realm). Each JSContext has its own global objects and system objects.
        // There can be several JSContexts per JSRuntime and they can share objects,
        // similar to frames of the same origin sharing Javascript objects in a web browser.
        let ctx = AsyncContext::full(&self.rt).await.unwrap();
        self.register_global(&ctx).await;
        self.run_in_context(&ctx, async |ctx: Ctx<'_>| {
            let (_, module_eval) = Module::evaluate_def::<OsModule, _>(ctx, "os")?;
            module_eval.into_future::<()>().await?;
            Ok(())
        })
        .await
        .unwrap();
        ctx
    }
    pub async fn call_code_args<'js, T: DeserializeOwned + 'static>(
        &self,
        f: &Function<'js>,
        args: Args<'js>,
    ) -> Result<T, Error> {
        // call js function with rust struct
        let v = f.call_arg::<Promise>(args)?.into_future::<Value>().await?;
        let from_type_name = v.type_name();
        let to_type_name = type_name::<T>();
        rquickjs_serde::from_value::<T>(v).map_err(|err| {
            Error::new_from_js_message(from_type_name, to_type_name, err.to_string())
        })
    }
    /// source must be esm
    pub async fn run_module_in_context<'js, T: DeserializeOwned + 'static>(
        &self,
        context: &AsyncContext,
        source: &str,
    ) -> Result<T, ErrorMessage> {
        let r = async_with!(context => |ctx| {
                let v = Module::evaluate(ctx,"vm",source)?.into_future::<Value>().await?;
                let from_type_name = v.type_name();
                let to_type_name = type_name::<T>();
                rquickjs_serde::from_value::<T>(v).map_err(|err|Error::new_from_js_message(from_type_name, to_type_name, err.to_string()))
        })
        .await;
        try_handle_error!(context, r)
    }
    /// run script source
    pub async fn eval_in_context<'js, T: DeserializeOwned + 'static>(
        &self,
        context: &AsyncContext,
        source: &str,
    ) -> Result<T, ErrorMessage> {
        let r = async_with!(context => |ctx| {
                let v= match ctx.eval::<Value, _>(source) {
                    Err(err)=> return Err(err),
                    Ok(v) => v,
                };
                let from_type_name = v.type_name();
                let to_type_name = type_name::<T>();
                rquickjs_serde::from_value::<T>(v).map_err(|err|Error::new_from_js_message(from_type_name, to_type_name, err.to_string()))
        })
        .await;
        try_handle_error!(context, r)
    }
}
fn handle_js_err(ctx: &Ctx<'_>, err: Error) -> ErrorMessage {
    match err {
        Error::Exception => {
            let e = ctx.catch();
            ErrorMessage {
                inner: err,
                message: Some(format!("{:?}", e)),
            }
        }
        err => ErrorMessage {
            inner: err,
            message: None,
        },
    }
}
