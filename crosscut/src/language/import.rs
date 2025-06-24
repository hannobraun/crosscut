use crate::Language;

use super::code::SyntaxNode;

pub fn import(code: &str) -> Language {
    let mut language = Language::new();

    let mut prev_indent = None;
    let mut parent_indents = Vec::new();

    for line in code.lines() {
        let Some(indent) = line.find(|ch: char| !ch.is_whitespace()) else {
            // Only whitespace on this line. Ignore it.
            continue;
        };

        handle_navigating_past_add_nodes(
            prev_indent,
            indent,
            &mut parent_indents,
            &mut language,
        );

        language.code(line.trim());
        language.down();

        prev_indent = Some(indent);
    }

    language
}

fn handle_navigating_past_add_nodes(
    prev_indent: Option<usize>,
    indent: usize,
    parent_indents: &mut Vec<usize>,
    language: &mut Language,
) {
    let Some(prev_indent) = prev_indent else {
        return;
    };

    if indent >= prev_indent {
        if indent > prev_indent {
            parent_indents.push(prev_indent);
        }

        return;
    }

    loop {
        let cursor = &language.editor().cursor().path;
        let current = language.codebase().node_at(cursor);

        let Some(parent_indent) = parent_indents.last().copied() else {
            assert!(
                !matches!(current.node, SyntaxNode::Add),
                "There are no parent nodes, so the current node can't be an \
                `Add`.",
            );

            // And if we're not at an `Add` node, then there's nothing to do for
            // this function.
            break;
        };

        if indent >= parent_indent {
            // This could not have been true in the first iteration of the loop,
            // as per the check above. Which means the code belove has done its
            // job and we're done.
            break;
        }

        if let SyntaxNode::Add = current.node {
            language.down();
        }

        parent_indents.pop();
    }
}
