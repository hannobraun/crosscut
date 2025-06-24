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

        loop {
            let cursor = &language.editor().cursor().path;
            let current = language.codebase().node_at(cursor);

            if let Some(prev_indent) = prev_indent {
                if indent >= prev_indent {
                    if indent > prev_indent {
                        indent_stack.push(prev_indent);
                    }

                    break;
                }
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

        language.code(line.trim());
        language.down();

        prev_indent = Some(indent);
    }

    language
}
