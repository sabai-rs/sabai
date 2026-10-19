use sabai::Result;
use sabai::console::Console;
use sabai::log::Logging;

fn main() -> Result<()> {
    let config = {{crate}}::config()?;
    Logging::new(&config.get_or("logging.level", String::from("info"))?)?.init()?;
    Console::new({{crate}}::app(config)?)
        .routes({{crate}}::routes())
        .run()
}
