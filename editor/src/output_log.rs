use std::sync::Mutex;

use imgui::Ui;

pub fn draw_log(ui: &Ui) {
    ui.window("Log")
        .build(|| {
            let mut log_data = LOG_DATA.lock().unwrap();
            for t in &log_data.log_lines {
                ui.text(t);
            }
            if log_data.new_line_pending {
                log_data.new_line_pending = false;
                ui.set_scroll_here_y_with_ratio(1.0);
            }
        });
}

struct LogData {
    new_line_pending: bool,
    log_lines: Vec<String>,
}
static LOG_DATA: Mutex<LogData> = Mutex::new(LogData { new_line_pending: false, log_lines: Vec::new() });

pub struct Logger;
impl log::Log for Logger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let mut log_data = LOG_DATA.lock().unwrap();
        log_data.log_lines.push(record.args().to_string());
        log_data.new_line_pending = true;
    }

    fn flush(&self) {}
}
