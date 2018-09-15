mod request;
mod response;
mod service;

pub use self::request::Request;
pub use self::response::Response;
pub use self::service::BaseService;

// ctx 是 box<Context>, impl middleware，impl call
// call(&mut self, req, ctx)-> Result<(resp, ctx), (err, req)>

// new()

// (req, ctx)
// before(mut req, mut ctx)-> Result<(req, ctx),(err,req)>

// (resp, ctx)
// after(resp, mut ctx)-> Result<(resp, ctx), (err,req)>

// if err, tokio will abort this connection
// error_handler(err,req)->Result<(), box<std::error::Error>>
