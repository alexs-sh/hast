mod logger;
mod recordsets;
mod server;
mod storage;

use structopt::StructOpt;
#[derive(Debug, StructOpt)]
#[structopt(name = "hast", about = "HAsh STorage")]
struct Options {
    #[structopt(
        short = "w",
        long = "workdir",
        default_value = "/tmp/hast/storage",
        about = "working directory"
    )]
    workdir: String,

    #[structopt(
        short = "s",
        long = "server",
        default_value = "0.0.0.0:8888",
        about = "address for binding server socket"
    )]
    server: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let options = Options::from_args();
    logger::setup();

    let storage = Box::new(storage::SimpleStorage::new(&options.workdir).init()?);
    let config = server::Config::new().with_address(&options.server);

    server::run(config, storage).await?;
    Ok(())
}
