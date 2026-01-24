use futures_util::{future::join_all, stream::FuturesUnordered};
use qck::run_in;
use rquickjs::{AsyncContext, AsyncRuntime};
use tokio::task::LocalSet;
mod ffi;
mod req;
mod setup;

#[tokio::main]
async fn main() {
    let rt = AsyncRuntime::new().unwrap();
    let local = LocalSet::new();
    local
        .run_until(async move {
            let futs = FuturesUnordered::new();
            for _ in 0..100 {
                // JSContext represents a Javascript context (or Realm). Each JSContext has its own global objects and system objects.
                // There can be several JSContexts per JSRuntime and they can share objects,
                // similar to frames of the same origin sharing Javascript objects in a web browser.
                let context = AsyncContext::full(&rt).await.unwrap();
                setup::setup_hook(&context).await;
                let ctx = context.clone();
                let task1 = tokio::task::spawn_local(async move { run_in(ctx).await });
                futs.push(task1);
            }
            let result = join_all(futs).await;
            for res in result {
                res.unwrap();
            }
        })
        .await;
}
