use log::{set_boxed_logger, set_max_level, Level, LevelFilter, Metadata, Record, SetLoggerError};
struct UnrealLogger;

impl log::Log for UnrealLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // TODO
        true
    }

    fn log(&self, record: &Record) {
        use std::ffi::CString;
        if self.enabled(record.metadata()) {
            //let text = CString::new(record.args().to_string()).unwrap();
            let text = record.args().to_string();
            (crate::module::bindings().log)(text.as_ptr() as *const _, text.len() as i32);
        }
    }

    fn flush(&self) {}
}

pub fn init() -> Result<(), SetLoggerError> {
    set_boxed_logger(Box::new(UnrealLogger)).map(|()| set_max_level(LevelFilter::Info))
}
