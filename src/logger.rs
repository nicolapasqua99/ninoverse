pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Danger
}

pub fn log(msg: String, level: LogLevel) {
    println!(msg.as_str());
}