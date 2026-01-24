use rquickjs::AsyncContext;

pub mod ffi;
pub mod req;
pub async fn run_in(ctx: AsyncContext) {
    ffi::run_js_func(ctx.clone()).await;
    ffi::run_on_request(ctx).await;
}
