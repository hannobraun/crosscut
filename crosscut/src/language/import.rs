use crate::Language;

use super::code::SyntaxNode;

pub fn import(code: &str) -> Language {
    let mut language = Language::new();

    let mut prev_indent = None;
    let mut indent_stack = Vec::new();

    for line in code.lines() {
        let Some(indent) = line.find(|ch: char| !ch.is_whitespace()) else {
            // Only whitespace on this line. Ignore it.
            continue;
        };

        if let Some(prev_indent) = prev_indent {
            update_indent_stack(
                &mut indent_stack,
                prev_indent,
                indent,
                &mut language,
            );
        }

        language.code(line.trim());
        language.down();

        prev_indent = Some(indent);
    }

    language
}

fn update_indent_stack(
    indent_stack: &mut Vec<usize>,
    prev_indent: usize,
    indent: usize,
    language: &mut Language,
) {
    loop {
        let cursor = &language.editor().cursor().path;
        let current = language.codebase().node_at(cursor);

        if indent >= prev_indent {
            if indent > prev_indent {
                indent_stack.push(prev_indent);
            }

            break;
        }

        let Some(parent_indent) = indent_stack.last().copied() else {
            break;
        };

        if indent >= parent_indent {
            indent_stack.pop();
            break;
        }

        if let SyntaxNode::Add = current.node {
            language.down();
        }

        indent_stack.pop();
    }
}
