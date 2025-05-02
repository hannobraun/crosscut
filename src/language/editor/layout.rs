use crate::language::code::{Codebase, LocatedNode, NodePath};

#[derive(Debug)]
pub struct EditorLayout {
    pub lines: Vec<EditorLine>,
}

impl EditorLayout {
    pub fn new(root: LocatedNode, codebase: &Codebase) -> Self {
        let mut nodes_from_root = Vec::new();
        collect_nodes_from_root(root, 0, &mut nodes_from_root, codebase);

        let lines = nodes_from_root
            .into_iter()
            .map(|node| {
                let level_of_indentation = node.distance_from_root;

                EditorLine {
                    node,
                    level_of_indentation,
                }
            })
            .collect();

        Self { lines }
    }

    pub fn node_before(&self, path: &NodePath) -> Option<&NodePath> {
        let line_index = self.line_index_of_node(path)?;

        let line_index_before = line_index.checked_sub(1)?;
        let line_before = self.lines.get(line_index_before)?;

        Some(&line_before.node.path)
    }

    pub fn node_after(&self, path: &NodePath) -> Option<&NodePath> {
        let line_index = self.line_index_of_node(path)?;

        let line_index_after = line_index.checked_add(1)?;
        let line_after = self.lines.get(line_index_after)?;

        Some(&line_after.node.path)
    }

    fn line_index_of_node(&self, path: &NodePath) -> Option<usize> {
        self.lines.iter().enumerate().find_map(|(index, line)| {
            (&line.node.path == path).then_some(index)
        })
    }
}

#[derive(Debug)]
pub struct EditorLine {
    pub node: NodeInLayout,
    pub level_of_indentation: u32,
}

impl EditorLine {
    pub fn width_of_indentation(&self) -> u32 {
        self.level_of_indentation
            * Self::NUMBER_OF_SPACES_PER_LEVEL_OF_INDENTATION
    }

    const NUMBER_OF_SPACES_PER_LEVEL_OF_INDENTATION: u32 = 4;
}

#[derive(Debug)]
pub struct NodeInLayout {
    pub path: NodePath,
    pub distance_from_root: u32,
}

fn collect_nodes_from_root(
    node: LocatedNode,
    distance_from_root: u32,
    nodes_from_root: &mut Vec<NodeInLayout>,
    codebase: &Codebase,
) {
    nodes_from_root.push(NodeInLayout {
        path: node.path.clone(),
        distance_from_root,
    });

    let mut children = Vec::new();
    children.extend(node.children(codebase.nodes()));

    for child in children {
        collect_nodes_from_root(
            child,
            distance_from_root + 1,
            nodes_from_root,
            codebase,
        );
    }
}
