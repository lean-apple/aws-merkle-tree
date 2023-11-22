use aws_merkle_tree::handler::handler;
use lambda_http::{run, service_fn, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(handler)).await
}
