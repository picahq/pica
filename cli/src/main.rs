use clap::Parser;
use picahq::{algebra::Handler, service::Pica};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Pica::parse();
    let context = args.load().await?;
    args.validate(&context).await?;
    args.run(&context).await?;

    Ok(())
}
