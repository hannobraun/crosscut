use std::collections::BTreeSet;

use super::{Body, Fragment, FragmentId, FragmentKind, Fragments};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Code {
    fragments: Fragments,

    pub root: Body,
    pub errors: BTreeSet<FragmentId>,
}

impl Code {
    pub fn fragments(&self) -> &Fragments {
        &self.fragments
    }

    pub fn push(&mut self, fragment: Fragment) -> FragmentId {
        let mut fragments_to_update = Vec::new();
        let mut body = &self.root;

        loop {
            let Some(id) = body.ids().next_back().copied() else {
                break;
            };

            let fragment @ Fragment {
                kind:
                    FragmentKind::Expression {
                        expression: Expression::FunctionCall { .. },
                    },
                ..
            } = self.fragments.get(&id)
            else {
                break;
            };

            fragments_to_update.push(id);
            body = &fragment.body;
        }

        let Some(to_update_id) = fragments_to_update.pop() else {
            return self.root.push(fragment, &mut self.fragments);
        };

        let mut to_update = self.fragments.get(&to_update_id).clone();
        let id_of_pushed = to_update.body.push(fragment, &mut self.fragments);

        let id_before_update = to_update_id;
        let updated = to_update;

        // We're missing code here that handles the rest of the fragments to
        // update. The test suite doesn't cover this yet, though, so I've
        // decided not to add it, for the time being.

        self.root
            .replace(id_before_update, updated, &mut self.fragments);

        id_of_pushed
    }
}

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Expression {
    FunctionCall { target: usize },
    LiteralValue { value: u32 },
}

#[derive(Clone, Debug, Eq, PartialEq, udigest::Digestable)]
pub enum Token {
    Identifier { name: String },
    LiteralNumber { value: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HostFunction {
    pub id: usize,
}
