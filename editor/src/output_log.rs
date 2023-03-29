use std::sync::Mutex;

use imgui::Ui;

pub fn draw_log(ui: &Ui) {
    ui.window("Log")
        .build(|| {
            let log_data = LOG_DATA.lock().unwrap();
            for t in &log_data.log_lines {
                ui.text(t);
            }
        });
}

struct LogData {
    log_lines: Vec<String>
}
static LOG_DATA: Mutex<LogData> = Mutex::new(LogData { log_lines: Vec::new() });

pub struct Logger;
impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let mut log_data = LOG_DATA.lock().unwrap();
        log_data.log_lines.push(record.args().to_string());
    }

    fn flush(&self) {}
}
