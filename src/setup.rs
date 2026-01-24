use rquickjs::function::Args;
use rquickjs::{
    AsyncContext, CatchResultExt, Exception, Function, Promise, Result, promise::Promised,
};
use rquickjs::{Class, Error, IntoJs};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::ffi;
use crate::req::{Request, SharedMap};
pub async fn run_js_func(context: AsyncContext) {
    context
        .async_with(async |ctx| {
            let ctx_clone = ctx.clone();
            let global = ctx.globals();
            let promised = Promised::from(async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                Result::<()>::Err(Exception::throw_message(&ctx_clone, "some_message"))
            });
            // call js function
            let foo: Function = global.get("foo").unwrap();

            match foo
                .call::<_, Promise>((promised,))
                .unwrap()
                .into_future::<()>()
                .await
            {
                Err(Error::Exception) => {
                    let e = ctx.catch();
                    panic!("{:?}", e);
                }
                Err(err) => panic!("{:?}", err),
                _ => {}
            }
        })
        .await
}

pub async fn run_on_request(context: AsyncContext) {
    context
        .async_with(async |ctx| {
            let global = ctx.globals();
            let on_req: Function = global.get("onRequest").unwrap();

            let mut m = HashMap::new();
            m.insert("from_rust_side".to_string(), "yes".to_string());

            let map = SharedMap(Arc::new(Mutex::new(m)));
            let req = Class::instance(ctx.clone(), Request::new_rust(map.clone())).unwrap();
            let mut args = Args::new(ctx.clone(), 1);
            let value = req.into_js(&ctx).unwrap();
            args.push_arg(value).unwrap();

            // call js function with rust struct
            on_req
                .call_arg::<Promise>(args)
                .unwrap()
                .into_future::<()>()
                .await
                .unwrap();
            // get final map
            assert!(map.0.lock().unwrap().get("from_js_side").is_some());
            assert!(map.0.lock().unwrap().get("from_rust_side").is_some());
        })
        .await
}
pub async fn setup_hook(context: &AsyncContext) {
    context
        .async_with(async |ctx| {
            let global = ctx.globals();
            global
                .set(
                    "__print",
                    Function::new(ctx.clone(), ffi::print)
                        .unwrap()
                        .with_name("__print")
                        .unwrap(),
                )
                .unwrap();
            let _f = ctx
                .eval::<Function, _>(
                    r"
                globalThis.foo = async function (v){
                    try{
                        await v;
                    }catch(e) {
                        if (e.message !== 'some_message'){
                            throw new Error('wrong error')
                        }
                        // call rust function
                        console.log(`call foo`)
                        return
                    }
                    throw new Error('no error thrown')
                }
                globalThis.onRequest = async function (req){
                    console.log(`111`)
                    req.setHeader(`from_js_side`,`yes`)
                    console.log(`222`)
                    console.log(req.toString())
                    console.log(`333`)
                }
            ",
                )
                .catch(&ctx)
                .unwrap();
            ctx.eval::<(), _>(
                r#"
                globalThis.console = {
                    log(...v) {
                        globalThis.__print(`${v.join(" ")}`)
                    }
                }
            "#,
            )
            .unwrap();
        })
        .await;
}
