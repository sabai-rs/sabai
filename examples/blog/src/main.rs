use sabai::Result;
use sabai::log::Logging;

fn main() -> Result<()> {
    let config = blog::config()?;
    Logging::new(&config.get_or("logging.level", String::from("info"))?)?.init()?;
    sabai::http::serve(blog::app(config)?, blog::routes())
}
