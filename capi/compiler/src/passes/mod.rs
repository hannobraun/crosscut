mod a_tail_position;
mod b_groups;
mod b_resolve;
mod d_fragments;
mod e_bytecode;

pub use {
    a_tail_position::determine_tail_positions, b_groups::group_functions,
    b_resolve::resolve_identifiers, d_fragments::generate_fragments,
    e_bytecode::generate_bytecode,
};
