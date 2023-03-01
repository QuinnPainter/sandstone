use std::path::PathBuf;
use std::num::NonZeroU32;
use dsengine_common::{SavedNodeGraph, SavedNode, SavedTransform, SavedNodeExtension};

pub struct ProjectData {
    pub path: PathBuf,
    pub name: String,
    pub graphs: Vec<crate::hierarchy::NodeGraph>
}

impl ProjectData {
    pub fn new() -> Self {
        Self {
            path: PathBuf::new(),
            name: String::new(),
            graphs: Vec::new()
        }
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
                    node_extension: SavedNodeExtension::None,
                    script_type_id: node.script_type_id,
                    enabled: node.enabled
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
