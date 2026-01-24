#[tokio::main]
async fn main() {
    println!("Hello World");
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use futures_util::{FutureExt, future::join_all, stream::FuturesUnordered};
    use qck::run_in;
    use rquickjs::{AsyncContext, AsyncRuntime};

    use qck::ffi;
    async fn run(rt: &AsyncRuntime) {
        let now = SystemTime::now();
        // JSContext represents a Javascript context (or Realm). Each JSContext has its own global objects and system objects.
        // There can be several JSContexts per JSRuntime and they can share objects,
        // similar to frames of the same origin sharing Javascript objects in a web browser.
        let context = AsyncContext::full(&rt).await.unwrap();
        ffi::setup_hook(&context).await;
        let ctx = context.clone();
        run_in(ctx).await;
        let after = SystemTime::now();
        println!(
            "time elapsed {}ms",
            after.duration_since(now).unwrap().as_millis()
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
}
