use rquickjs::{Ctx, Error};

pub mod ffi;
pub mod req;

pub fn handle_js_err(ctx: &Ctx<'_>, err: Error) {
    match err {
        Error::Exception => {
            let e = ctx.catch();
            panic!("{:?}", e);
        }
        err => panic!("{:?}", err),
    }
}
