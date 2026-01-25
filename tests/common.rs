use qck::{handle_js_err, req};
use rquickjs::{AsyncContext, Exception, Function, Promise, Result, promise::Promised};

pub async fn run_test_code(ctx: AsyncContext) {
    call_js_and_call_rust_from_js_cb(ctx.clone()).await;
    req::run_on_request(ctx).await;
}

pub async fn call_js_and_call_rust_from_js_cb(context: AsyncContext) {
    context
        .async_with(async |ctx| {
            let ctx_clone = ctx.clone();
            let global = ctx.globals();
            let promised = Promised::from(async move {
                // tokio::time::sleep(Duration::from_millis(100)).await;
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
                Err(err) => handle_js_err(&ctx, err),
                _ => {}
            }
        })
        .await
}

pub const SETUP_CODE: &str = r#"
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
