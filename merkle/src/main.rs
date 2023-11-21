use lambda_http::{
    run, service_fn, Error,
};
use aws_merkle_tree::handler::handler;

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(handler)).await
}
