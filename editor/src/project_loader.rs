use std::sync::mpsc;
use std::thread;
use std::path::{Path, PathBuf};
use imgui::Ui;

pub struct ProjectLoader {
    new_project_path_buffer: String,
    new_project_name_buffer: String,
    file_dialog_transmitter: mpsc::Sender<FileDialogReturnInfo>,
    file_dialog_receiver: mpsc::Receiver<FileDialogReturnInfo>,
    open_load_project_modal: bool
}

impl ProjectLoader {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<FileDialogReturnInfo>();
        Self {
            new_project_path_buffer: String::new(),
            new_project_name_buffer: String::new(),
            file_dialog_transmitter: tx,
            file_dialog_receiver: rx,
            open_load_project_modal: true
        }
    }

    pub fn open_load_project_modal(&mut self) {
        self.open_load_project_modal = true;
    }

    fn new_project_file_dialog(&self) {
        let tx = self.file_dialog_transmitter.clone();
        thread::spawn(move || {
            let path = tinyfiledialogs::select_folder_dialog("New Project", "");
            tx.send(FileDialogReturnInfo::NewProject(path)).unwrap();
        });
    }
    
    pub fn open_project_file_dialog(&self) {
        let tx = self.file_dialog_transmitter.clone();
        thread::spawn(move || {
            let path = tinyfiledialogs::select_folder_dialog("Open Project", "");
            tx.send(FileDialogReturnInfo::OpenProject(path)).unwrap();
        });
    }

    pub fn update(&mut self, ui: &Ui) {
        ui.modal_popup_config("Load Project").resizable(false).always_auto_resize(true).build(|| {
            if let Some(tab_bar_token) = ui.tab_bar("tabs") {
                if let Some(tab_token) = ui.tab_item("New") {
                    ui.text("Project name");
                    ui.input_text("##ProjectName", &mut self.new_project_name_buffer)
                        .callback(imgui::InputTextCallback::CHAR_FILTER, FileNameInputFilter).build();
                    ui.text("Location");
                    ui.input_text("##PathInput", &mut self.new_project_path_buffer).build();
                    ui.same_line();
                    if ui.button("Browse") {
                        self.new_project_file_dialog();
                    }
                    ui.spacing();
                    // Enable text wrapping using the current window width
                    let text_wrap_token = ui.push_text_wrap_pos();
                    let total_path = Path::new(&self.new_project_path_buffer).join(&self.new_project_name_buffer);
                    ui.text_disabled(format!("Project will be created in {}", total_path.display()));
                    text_wrap_token.end();
                    ui.spacing();
                    if ui.button("Create") {
                        create_new_project(total_path);
                        ui.close_current_popup();
                    }
                    tab_token.end();
                }
                if let Some(tab_token) = ui.tab_item("Open") {
                    if ui.button_with_size("Open", [90.0, 30.0]) {
                        self.open_project_file_dialog();
                    }
                    tab_token.end();
                }
                tab_bar_token.end();
            }
        });

        // Workaround for https://github.com/ocornut/imgui/issues/331
        if self.open_load_project_modal {
            // Set starting path to the user's home directory
            self.new_project_path_buffer = String::from(
                home::home_dir().unwrap_or(PathBuf::new())
                .to_str().unwrap_or(""));
            ui.open_popup("Load Project");
            self.open_load_project_modal = false;
        }

        // Handle the file dialog close event
        if let Ok(dialog_info) = self.file_dialog_receiver.try_recv() {
            match dialog_info {
                FileDialogReturnInfo::NewProject(Some(path)) => {
                    self.new_project_path_buffer = path;
                }
                FileDialogReturnInfo::OpenProject(Some(path)) => {}
                _ => {}
            }
        }
    }
}
    
fn create_new_project(path: PathBuf) {
    // todo: handle IO errors
    std::fs::create_dir_all(path).unwrap();
}


enum FileDialogReturnInfo {
    NewProject(Option<String>),
    OpenProject(Option<String>)
}

struct FileNameInputFilter;
impl imgui::InputTextCallbackHandler for FileNameInputFilter {
    fn char_filter(&mut self, c: char) -> Option<char> {
        // Characters that are problematic for file names.
        // https://en.wikipedia.org/wiki/Filename#Reserved_characters_and_words
        // Could be more restrictive and change this to a whitelist.
        const INVALID_CHARS: [char; 11] = ['/', '\\', '?', '%', '*', '*', ':', '|', '"', '<', '>'];
        if INVALID_CHARS.contains(&c) {
            None
        } else {
            Some(c)
        }
    }
}
