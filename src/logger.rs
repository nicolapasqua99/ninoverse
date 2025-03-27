pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Danger,
}

pub fn log(msg: String, level: LogLevel) {
    match level {
        LogLevel::Debug => println!("{}", msg),
        LogLevel::Info => println!("{}", msg),
        LogLevel::Warning => println!("{}", msg),
        LogLevel::Danger => println!("{}", msg),
    }
}
