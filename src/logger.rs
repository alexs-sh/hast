use env_logger::Builder;
use log::LevelFilter;

pub fn setup() {
    let mut builder = Builder::new();
    builder.filter_level(LevelFilter::Info);
    builder.parse_default_env();
    builder.init();
}
