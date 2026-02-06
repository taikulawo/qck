use std::time::SystemTime;

use futures_util::{FutureExt, future::join_all, stream::FuturesUnordered};
mod common;
use qck::JsEngine;

use crate::common::{SETUP_CODE, run_test_code_in_context};
async fn run() {
    let rt = JsEngine::new().await.unwrap();
    let now = SystemTime::now();
    let context = rt.new_context().await;
    rt.run_module_in_context::<()>(&context, SETUP_CODE)
        .await
        .unwrap();
    run_test_code_in_context(&rt, &context).await.unwrap();
    let after = SystemTime::now();
    println!(
        "time elapsed {} us",
        after.duration_since(now).unwrap().as_micros()
    )
}

#[tokio::test]
async fn test_in_single_tokio_thread() {
    run().await;
}
#[tokio::test]
async fn test_in_single_tokio_thread_loop() {
    let futs = FuturesUnordered::new();
    for _ in 0..100 {
        let fut = run().boxed_local();
        futs.push(fut);
    }
    let _ = join_all(futs).await;
}
