use qck::{ErrorMessage, JsEngine};
use rquickjs::{AsyncContext, Ctx, Error, Exception, Function, function::Args, promise::Promised};
#[allow(unused)]
pub async fn run_test_code(rt: &JsEngine) -> Result<(), ErrorMessage> {
    let context = rt.new_context().await;
    rt.run_module_in_context::<()>(&context, SETUP_CODE)
        .await
        .unwrap();
    run_test_code_in_context(rt, &context).await
}
pub async fn run_test_code_in_context(
    rt: &JsEngine,
    context: &AsyncContext,
) -> Result<(), ErrorMessage> {
    call_js_and_call_rust_from_js_cb(&context, rt).await?;
    rt.run_on_request::<()>(&context).await
}
pub async fn call_js_and_call_rust_from_js_cb(
    context: &AsyncContext,
    rt: &JsEngine,
) -> Result<(), ErrorMessage> {
    rt.run_in_context(&context, async |ctx: Ctx<'_>| {
        let ctx_clone = ctx.clone();
        let global = ctx.globals();
        let promised = Promised::from(async move {
            // tokio::time::sleep(Duration::from_millis(100)).await;
            Result::<(), Error>::Err(Exception::throw_message(&ctx_clone, "some_message"))
        });
        // call js function
        let foo: Function = global.get("foo").unwrap();
        let mut args = Args::new(ctx.clone(), 1);
        args.push_arg(promised).unwrap();
        rt.call_code_args::<()>(&foo, args).await
    })
    .await
}

#[allow(unused)]
pub const SETUP_CODE: &str = r#"
     import { type } from "os";

    (function(){globalThis.console = {
        log(...v) {
            globalThis.__ffi_print(`${v.join(" ")}`)
        }
    }
    globalThis.foo = async function (v){
        try{
            await v;
        }catch(e) {
            if (e.message !== 'some_message'){
                throw new Error('wrong error')
            }
            return
        }
        throw new Error('no error thrown')
    }
    globalThis.onRequest = async function (req){
        req.setHeader(`from_js_side`,`yes`)
    }})()
"#;
