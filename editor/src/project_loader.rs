use std::sync::mpsc;
use std::thread;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Write;
use std::num::{NonZeroU32, NonZeroUsize};
use imgui::Ui;
use serde::{Serialize, Deserialize};
use crate::project_data::ProjectData;
use crate::hierarchy::{NodeGraph, Node, Transform, Hierarchy};

pub struct ProjectLoader {
    new_project_path_buffer: String,
    new_project_name_buffer: String,
    file_dialog_transmitter: mpsc::Sender<FileDialogReturnInfo>,
    file_dialog_receiver: mpsc::Receiver<FileDialogReturnInfo>,
    open_load_project_modal: bool,
    close_load_project_modal: bool
}

impl ProjectLoader {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<FileDialogReturnInfo>();
        Self {
            new_project_path_buffer: String::new(),
            new_project_name_buffer: String::new(),
            file_dialog_transmitter: tx,
            file_dialog_receiver: rx,
            open_load_project_modal: true,
            close_load_project_modal: false
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

    pub fn update(&mut self, ui: &Ui, project_data: &mut ProjectData, hierarchy: &mut Hierarchy) {
        // Draw the "Load Project" popup modal
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
                        create_new_project(&total_path, self.new_project_name_buffer.clone(), project_data, hierarchy);
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
                if self.close_load_project_modal {
                    self.close_load_project_modal = false;
                    ui.close_current_popup();
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
                FileDialogReturnInfo::OpenProject(Some(path)) => {
                    load_project(Path::new(&path), project_data, hierarchy);
                    self.close_load_project_modal = true;
                }
                _ => {}
            }
        }
    }
}
    
fn create_new_project(path: &Path, name: String, project_data: &mut ProjectData, hierarchy: &mut Hierarchy) {
    project_data.name = name;
    project_data.path = path.to_path_buf();
    project_data.graphs = Vec::new();
    save_project(project_data);
    load_project(path, project_data, hierarchy);
}

fn load_project(path: &Path, project_data: &mut ProjectData, hierarchy: &mut Hierarchy) {
    // todo: handle IO errors
    let project_data_file = File::open(path.join("project_info.ron")).unwrap();
    let saved_project_data: SavedProjectData = ron::de::from_reader(project_data_file).unwrap();

    project_data.path = path.to_path_buf();
    project_data.name = saved_project_data.name;
    project_data.graphs.clear();
    project_data.graphs.reserve(saved_project_data.prefabs.len());
    for graph in saved_project_data.prefabs {
        let mut new_graph = NodeGraph::new();
        for node in graph.nodes {
            new_graph.0.push(Node {
                child_index: node.child_index.map(nzu32_to_nzusize),
                parent_index: node.parent_index.map(|x| x as usize),
                sibling_index: node.sibling_index.map(nzu32_to_nzusize),
                name: node.name,
                transform: Transform { x: node.transform.x, y: node.transform.y },
                script_type_id: node.script_type_id,
                enabled: node.enabled
            });
        }
        project_data.graphs.push(new_graph);
    }
    hierarchy.current_graph_idx = 0;
    hierarchy.selected_node_idx = None;
}

pub fn save_project(project_data: &mut ProjectData) {
    // todo: handle IO errors
    std::fs::create_dir_all(&project_data.path).unwrap();
    
    let saved_project_data = SavedProjectData {
        name: project_data.name.clone(),
        prefabs: project_data.export_saved_graph()
    };
    let ser_project_data = ron::ser::to_string_pretty(&saved_project_data, ron::ser::PrettyConfig::default()).unwrap();

    let mut project_data_file = File::create(project_data.path.join("project_info.ron")).unwrap();
    project_data_file.write_all(ser_project_data.as_bytes()).unwrap();
}

#[inline(always)]
fn nzu32_to_nzusize(x: NonZeroU32) -> NonZeroUsize {
    // This is fully safe, as new_unchecked only fails if input is 0 - the input is NonZero
    unsafe { NonZeroUsize::new_unchecked(u32::from(x) as usize) }
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

#[derive(Serialize, Deserialize)]
pub struct SavedProjectData {
    name: String,
    prefabs: Vec<dsengine_common::SavedNodeGraph>
}
