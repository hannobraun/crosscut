use crate::language::{
    code::{CodeError, Codebase, NodeKind},
    compiler::Compiler,
    packages::Packages,
};

#[test]
fn empty_node_with_multiple_children_is_an_error() {
    // An empty node has rather obvious runtime semantics: Do nothing and just
    // pass on the active value unchanged.
    //
    // If an empty node has multiple children, then it's no longer obvious what
    // it should do. So that needs to be an error.

    let packages = Packages::new();

    let mut codebase = Codebase::new();
    let mut compiler = Compiler::new(&mut codebase);

    // Verify the assumptions this tests makes about the default root node.
    assert_eq!(compiler.codebase().root().node.kind(), &NodeKind::Empty);

    compiler.insert_child(&compiler.codebase().root().path, "", &packages);
    compiler.insert_child(&compiler.codebase().root().path, "", &packages);

    assert_eq!(
        compiler.codebase().root().node.kind(),
        &NodeKind::Error {
            node: "".to_string()
        },
    );

    let error = compiler.codebase().root().path;
    assert_eq!(
        compiler.codebase().error_at(&error),
        Some(&CodeError::EmptyNodeWithMultipleChildren),
    );
}
