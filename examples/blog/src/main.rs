use sabai::Result;
use sabai::console::Console;
use sabai::log::Logging;

fn main() -> Result<()> {
    let config = blog::config()?;
    Logging::new(&config.get_or("logging.level", String::from("info"))?)?.init()?;
    Console::new(blog::app(config)?)
        .routes(blog::routes())
        .migrations(blog::migrations())
        .run()
}
