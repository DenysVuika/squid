use env_logger::{Builder, Env};
use std::io::Write;

pub fn init(log_level: Option<&str>) {
    let default_level = log_level.unwrap_or("info");

    let env = Env::default()
        .filter_or("LOG_LEVEL", default_level)
        .write_style_or("LOG_STYLE", "always");

    // env_logger::init_from_env(env);

    Builder::from_env(env)
        .format(|buf, record| {
            let level = record.level();
            let info_style = buf.default_level_style(record.level());
            // let timestamp = buf.timestamp();
            // writeln!(buf, "{level}: {info_style}{}{info_style:#}", record.args())
            writeln!(buf, "{info_style}{level}: {info_style:#}{}", record.args())
        })
        .init();
}
