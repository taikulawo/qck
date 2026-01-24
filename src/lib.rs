use rquickjs::AsyncContext;

pub mod ffi;
pub mod req;
pub mod setup;
pub async fn run_in(ctx: AsyncContext) {
    setup::run_js_func(ctx.clone()).await;
    setup::run_on_request(ctx).await;
}
