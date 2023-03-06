use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::num::NonZeroU32;
use notify::Watcher;
use dsengine_common::{SavedNodeGraph, SavedNode, SavedTransform};

pub struct ProjectData {
    path: PathBuf,
    pub name: String,
    pub graphs: Vec<crate::hierarchy::NodeGraph>,
    file_scanner_tx: std::sync::mpsc::Sender<Result<notify::Event, notify::Error>>,
    file_scanner_rx: std::sync::mpsc::Receiver<Result<notify::Event, notify::Error>>,
    file_scanner_watcher: Option<notify::INotifyWatcher>,
    pub graphical_assets: HashMap<String, PathBuf>,
}

impl ProjectData {
    pub fn new() -> Self {
        let (tx, rx) =  std::sync::mpsc::channel();
        Self {
            path: PathBuf::new(),
            name: String::new(),
            graphs: Vec::new(),
            file_scanner_tx: tx,
            file_scanner_rx: rx,
            file_scanner_watcher: None,
            graphical_assets: HashMap::new(),
        }
    }

    pub fn set_path_without_watch(&mut self, path: PathBuf) {
        self.path = path;
    }

    pub fn set_path(&mut self, path: PathBuf) {
        self.path = path;
        // recreate watcher to clear previously watched paths
        self.file_scanner_watcher = Some(notify::RecommendedWatcher::new(
            self.file_scanner_tx.clone(),
            notify::Config::default()).unwrap());
        self.file_scanner_watcher
            .as_mut()
            .unwrap()
            .watch(&self.path, notify::RecursiveMode::Recursive)
            .unwrap();
    }

    pub fn get_path(&self) -> &Path {
        &self.path
    }

    pub fn check_file_scanner(&mut self) {
        let mut any_changes = false;
        // Iterate over receiver to clear queue
        for _ in self.file_scanner_rx.try_iter() { any_changes = true; }
        if any_changes {
            self.find_graphical_assets();
        }
    }

    pub fn find_graphical_assets(&mut self) {
        let asset_path = self.path.join("assets");
        self.graphical_assets.clear();
    
        for entry in asset_path.read_dir().unwrap() {
            let entry_path = entry.unwrap().path();
            if let Some(extension) = entry_path.extension() {
                if extension == "png" {
                    let file_name = entry_path.with_extension("").file_name().unwrap().to_str().unwrap().to_string();
                    self.graphical_assets.insert(file_name, entry_path);
                }
            }
        }
        log::info!("Found graphical assets: {:?}", self.graphical_assets);
    }

    pub fn export_saved_graph(&self) -> Vec<SavedNodeGraph> {
        let mut all_saved_graphs: Vec<SavedNodeGraph> = Vec::with_capacity(self.graphs.len());
        let mut old_indices: Vec<usize> = Vec::new();
        for graph in &self.graphs {
            old_indices.clear();
            old_indices.resize(graph.0.find_last_index().map_or(0, |x| x + 1), 0);
            let mut saved_graph = SavedNodeGraph { nodes: Vec::with_capacity(graph.0.num_elements()) };
    
            // Create the nodes with placeholder indices
            for (i, node) in &graph.0 {
                old_indices[i] = saved_graph.nodes.len();
                saved_graph.nodes.push(SavedNode {
                    child_index: None,
                    parent_index: None,
                    sibling_index: None,
                    name: node.name.clone(),
                    transform: SavedTransform { x: node.transform.x, y: node.transform.y },
                    node_extension: node.node_extension.to_saved(),
                    script_type_id: node.script_type_id,
                    enabled: node.enabled,
                });
            }
    
            // Wire up the child, parent and sibling relations with the new indices
            for (snode, (_, node)) in saved_graph.nodes.iter_mut().zip(graph.0.iter()) {
                snode.child_index = node.child_index.map(|x| NonZeroU32::new(old_indices[usize::from(x)] as u32).unwrap());
                snode.parent_index = node.parent_index.map(|x| old_indices[x] as u32);
                snode.sibling_index = node.sibling_index.map(|x| NonZeroU32::new(old_indices[usize::from(x)] as u32).unwrap());
            }
            all_saved_graphs.push(saved_graph);
        }
        all_saved_graphs
    }
}
