use std::time::SystemTime;

use futures_util::{FutureExt, future::join_all, stream::FuturesUnordered};
mod common;
use common::run_test_code;
use qck::ffi;
use rquickjs::{AsyncContext, AsyncRuntime, loader::ModuleLoader};
use rquickjs_extra::os::OsModule;

use crate::common::SETUP_CODE;
async fn run(rt: &AsyncRuntime) {
    // TODO 加载平台API
    // let loader = (ModuleLoader::default().with_module("os", OsModule),);

    // JSContext represents a Javascript context (or Realm). Each JSContext has its own global objects and system objects.
    // There can be several JSContexts per JSRuntime and they can share objects,
    // similar to frames of the same origin sharing Javascript objects in a web browser.
    let context = AsyncContext::full(&rt).await.unwrap();
    ffi::setup_runtime_context(&context, SETUP_CODE).await;
    let now = SystemTime::now();
    let ctx = context.clone();
    run_test_code(ctx).await;
    let after = SystemTime::now();
    println!(
        "time elapsed {} us",
        after.duration_since(now).unwrap().as_micros()
    )
}

#[tokio::test]
async fn test_in_single_tokio_thread() {
    let rt = AsyncRuntime::new().unwrap();
    run(&rt).await;
}
#[tokio::test]
async fn test_in_single_tokio_thread_loop() {
    let rt = AsyncRuntime::new().unwrap();
    let futs = FuturesUnordered::new();
    for _ in 0..100 {
        let fut = run(&rt).boxed_local();
        futs.push(fut);
    }
    let _ = join_all(futs).await;
}
