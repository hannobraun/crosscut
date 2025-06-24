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
            &mut parent_indents,
            prev_indent,
            indent,
            &mut language,
        );

        language.code(line.trim());
        language.down();

        prev_indent = Some(indent);
    }

    language
}

fn handle_navigating_past_add_nodes(
    parent_indents: &mut Vec<usize>,
    prev_indent: Option<usize>,
    indent: usize,
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
            break;
        };

        if indent >= parent_indent {
            parent_indents.pop();
            break;
        }

        if let SyntaxNode::Add = current.node {
            language.down();
        }

        parent_indents.pop();
    }
}
