use std::collections::{BTreeMap, BTreeSet};

use super::{NodeHash, NodePath, Nodes};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Changes {
    change_sets: Vec<ChangeSet>,
}

impl Changes {
    pub fn new_change_set<'r>(
        &'r mut self,
        root_before_change: NodeHash,
        nodes: &'r mut Nodes,
    ) -> NewChangeSet<'r> {
        self.change_sets.push(ChangeSet {
            replacements_by_replaced: BTreeMap::new(),
        });

        let Some(change_set) = self.change_sets.last_mut() else {
            unreachable!("Just pushed a change set. One _must_ be available.");
        };

        NewChangeSet {
            change_set,
            root_before_change,
            nodes,
        }
    }

    pub fn latest_version_of<'r>(&'r self, path: &'r NodePath) -> &'r NodePath {
        let Some(i) = self.change_sets.iter().enumerate().rev().find_map(
            |(i, change_set)| {
                change_set
                    .replacements_by_replaced
                    .contains_key(path)
                    .then_some(i)
            },
        ) else {
            return path;
        };

        let mut latest_known = path;

        for change_set in &self.change_sets[i..] {
            let Ok(latest) = change_set.latest_version_of(latest_known) else {
                unreachable!(
                    "Detected circular replacement path in change set. This \
                    should be impossible, as this case is checked below, when \
                    making the replacement."
                );
            };

            latest_known = latest;
        }

        latest_known
    }
}

#[derive(Debug)]
pub struct NewChangeSet<'r> {
    change_set: &'r mut ChangeSet,
    #[cfg_attr(not(test), allow(unused))]
    root_before_change: NodeHash,

    pub nodes: &'r mut Nodes,
}

impl NewChangeSet<'_> {
    pub fn change_set(&self) -> &ChangeSet {
        self.change_set
    }

    #[cfg(test)]
    pub fn root_before_change(&self) -> NodePath {
        NodePath::for_root(self.root_before_change)
    }

    /// # Mark a node in the change set as replacing another
    ///
    /// This method only tracks the replacement of nodes. It doesn't insert them
    /// itself.
    ///
    /// It it the responsibility of the caller to insert the new node, then
    /// construct a [`NodePath`] for it, using the contextual information it has
    /// access to.
    ///
    /// ## Panics
    ///
    /// Panics, if this replacement would create a cycle of replacements within
    /// this change set. For example if `A` was marked as being replaced by `B`,
    /// then `B` as being replaced by `A`.
    ///
    /// While such a cycle is perfectly fine, if spread over multiple change
    /// sets, it must not occur within a single change set. This case would be
    /// considered a bug in the caller of this method.
    pub fn replace(&mut self, to_replace: &NodePath, replacement: &NodePath) {
        if replacement != to_replace {
            // Nodes are "replaced" by identical ones all the time. Making the
            // caller responsible for checking that would be onerous.
            //
            // And that is generally not a problem. But inserting such a
            // replacement into the change set, would confuse the code that
            // looks for the latest version of a node.

            self.change_set
                .replacements_by_replaced
                .insert(to_replace.clone(), replacement.clone());
        }

        if self.change_set.latest_version_of(replacement).is_err() {
            panic!(
                "You must not create a cycle of replacements within a single \
                change set."
            );
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangeSet {
    replacements_by_replaced: BTreeMap<NodePath, NodePath>,
}

impl ChangeSet {
    pub fn was_replaced(&self, replaced: &NodePath) -> Option<&NodePath> {
        self.replacements_by_replaced.get(replaced)
    }

    fn latest_version_of<'r>(
        &'r self,
        path: &'r NodePath,
    ) -> Result<&'r NodePath, CircularDependency> {
        let mut already_seen = BTreeSet::new();
        let mut latest_known = path;

        while let Some(later) = self.replacements_by_replaced.get(latest_known)
        {
            already_seen.insert(latest_known);

            if already_seen.contains(later) {
                return Err(CircularDependency);
            } else {
                latest_known = later;
            }
        }

        Ok(latest_known)
    }
}

struct CircularDependency;

#[cfg(test)]
mod tests {
    use crate::language::code::{NodePath, Nodes, SyntaxNode};

    use super::Changes;

    #[test]
    fn circular_changes_should_work_correctly() {
        let mut changes = Changes::default();
        let mut nodes = Nodes::default();

        let [node_a, node_b] =
            ["a", "b"].map(|identifier| SyntaxNode::Identifier {
                name: String::from(identifier),
            });
        let path_a = {
            let hash = nodes.insert(node_a.clone());
            NodePath::for_root(hash)
        };

        let path_b = {
            let mut change_set =
                changes.new_change_set(*path_a.hash(), &mut nodes);

            let path_b = NodePath::for_root(change_set.nodes.insert(node_b));
            change_set.replace(&path_a, &path_b);

            path_b
        };
        let path_a = {
            let mut change_set =
                changes.new_change_set(*path_b.hash(), &mut nodes);

            let path_a = NodePath::for_root(change_set.nodes.insert(node_a));
            change_set.replace(&path_b, &path_a);

            path_a
        };

        assert_eq!(changes.latest_version_of(&path_a), &path_a);
        assert_eq!(changes.latest_version_of(&path_b), &path_a);
    }
}
