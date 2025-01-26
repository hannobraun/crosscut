use crosscut_compiler::code::syntax::MemberLocation;

#[derive(Clone)]
pub enum UserAction {
    BreakpointClear { expression: MemberLocation },
    BreakpointSet { expression: MemberLocation },
    Continue,
    Reset,
    StepIn,
    StepOut,
    StepOver,
    Stop,
}
