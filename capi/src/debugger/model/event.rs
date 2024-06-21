use crate::runtime;

#[derive(Clone, Debug)]
pub enum DebugCommand {
    BreakpointClear {
        location: runtime::Location,
    },
    BreakpointSet {
        location: runtime::Location,
    },
    Continue {
        and_stop_at: Option<runtime::Location>,
    },
    Reset,
    Step,
    Stop,
}
