use std::time::SystemTime;

use futures_util::{FutureExt, future::join_all, stream::FuturesUnordered};
mod common;
use common::run_test_code;
use qck::JsEngine;

use crate::common::SETUP_CODE;
async fn run() {
    let rt = JsEngine::new().await.unwrap();
    let now = SystemTime::now();
    rt.eval_in_new_context::<()>(SETUP_CODE).await.unwrap();
    run_test_code(&rt).await.unwrap();
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
