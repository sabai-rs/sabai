use sabai::Result;
use sabai::console::Console;
use sabai::log::Logging;

fn main() -> Result<()> {
    let config = bootstrap::config()?;
    Logging::new(&config.get_or("logging.level", String::from("info"))?)?.init()?;
    Console::new(bootstrap::app(config)?)
        .routes(bootstrap::routes())
        .run()
}
