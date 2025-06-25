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

        handle_add_nodes(
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

fn handle_add_nodes(
    prev_indent: Option<usize>,
    indent: usize,
    parent_indents: &mut Vec<usize>,
    language: &mut Language,
) {
    if let Some(prev_indent) = prev_indent {
        if indent >= prev_indent {
            if indent > prev_indent {
                parent_indents.push(prev_indent);
            }

            return;
        }
    };

    navigate_past_add_nodes(indent, parent_indents, language);
}

fn navigate_past_add_nodes(
    indent: usize,
    parent_indents: &mut Vec<usize>,
    language: &mut Language,
) {
    while let Some(parent_indent) = parent_indents.last().copied() {
        assert!(indent <= parent_indent, "Loop should be done already.");

        if indent == parent_indent {
            // This could not have been true in the first iteration of the loop,
            // as per the check above. Which means the code belove has done its
            // job and we're done.
            break;
        }

        let cursor = &language.editor().cursor().path;
        let node = language.codebase().nodes().get(cursor.hash());

        if let SyntaxNode::Add = node {
            language.down();
        }

        parent_indents.pop();
    }
}
