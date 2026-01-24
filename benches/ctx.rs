use criterion::{Criterion, criterion_group, criterion_main};
use qck::{ffi::setup_hook, run_in};
use rquickjs::{AsyncContext, AsyncRuntime};
use std::hint::black_box;

fn bench_js(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let _guard = rt.enter();
    let js_rt = AsyncRuntime::new().unwrap();

    {
        let context = rt.block_on(async {
            let context = AsyncContext::full(&js_rt).await.unwrap();
            setup_hook(&context).await;
            context
        });
        c.bench_function("use same context", |b| {
            b.to_async(&rt).iter(|| black_box(run_in(context.clone())))
        });
    }
    c.bench_function("use separate context", |b: &mut criterion::Bencher<'_>| {
        b.to_async(&rt).iter(|| {
            black_box(async {
                let context = AsyncContext::full(&js_rt).await.unwrap();
                setup_hook(&context).await;
                run_in(context).await;
            })
        })
    });
}
fn custom_config() -> Criterion {
    Criterion::default()
}

criterion_group!(name = benches;config = custom_config();targets = bench_js);
criterion_main!(benches);
