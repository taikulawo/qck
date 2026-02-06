use std::time::Instant;

use criterion::{Criterion, criterion_group, criterion_main};
use qck::JsEngine;
use rquickjs::{AsyncContext, AsyncRuntime};
use tokio::task::LocalSet;

use crate::common::{SETUP_CODE, run_test_code, run_test_code_in_context};
#[path = "../tests/common.rs"]
mod common;
// 结论：
// AsyncRuntime单次创建 10us
// AsyncContext 单次创建 150us
// 但总体 150微妙 可接受
async fn setup() -> (JsEngine, AsyncContext) {
    let engine = JsEngine::new().await.unwrap();
    let context = engine.new_context().await;
    engine
        .eval_in_context::<()>(&context, SETUP_CODE)
        .await
        .unwrap();
    (engine, context)
}
fn bench_js_call(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let _guard = rt.enter();
    {
        c.bench_function("use same context", |b| {
            b.to_async(&rt).iter_custom(|iter| async move {
                let (rt, context) = setup().await;
                let start = Instant::now();
                for _ in 0..iter {
                    run_test_code_in_context(&rt, &context).await.unwrap();
                }
                start.elapsed()
            })
        });
    }
    {
        c.bench_function("use same context and parallel run task", |b| {
            b.to_async(&rt).iter_custom(|iter| async move {
                let (rt, context) = setup().await;

                let local = LocalSet::new();
                let start = Instant::now();
                for _ in 0..iter {
                    run_test_code_in_context(&rt, &context).await.unwrap();
                }
                local.await;
                start.elapsed()
            })
        });
    }
    async fn run_in_new_context(rt: &JsEngine) {
        run_test_code(rt).await.unwrap();
    }
    c.bench_function("use separate context run test code", |b| {
        b.to_async(&rt).iter_custom(|iter| async move {
            let start = Instant::now();
            let (rt, _) = setup().await;
            for _ in 0..iter {
                run_in_new_context(&rt).await;
            }
            start.elapsed()
        })
    });
    c.bench_function("use separate context and parallel run task", |b| {
        b.to_async(&rt).iter_custom(|iter| async move {
            let local = LocalSet::new();
            let start = Instant::now();
            let rt = setup().await;
            for _ in 0..iter {
                let (rt, _) = rt.clone();
                local.spawn_local(async move {
                    run_in_new_context(&rt).await;
                });
            }
            local.await;
            start.elapsed()
        })
    });
}

fn bench_async_runtime_create(c: &mut Criterion) {
    c.bench_function("bench create AsyncRuntime", |b| {
        b.iter(|| AsyncRuntime::new().unwrap())
    });
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    c.bench_function("bench init AsyncRuntime/AsyncContext", |b| {
        b.to_async(&rt).iter(|| async { setup().await })
    });
}
fn custom_config() -> Criterion {
    Criterion::default()
}

criterion_group!(name = benches;config = custom_config();targets = bench_js_call, bench_async_runtime_create);
criterion_main!(benches);
