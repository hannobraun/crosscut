use capi_runtime::{Effect, InstructionAddress, Value};

/// # The current state of the runtime
#[derive(Clone, Debug)]
pub enum RuntimeState {
    /// # The process is currently running
    Running,

    /// # The process has finished
    Finished,

    /// # The process is currently stopped
    Stopped {
        /// # The queue of unhandled effects
        effects: Vec<Effect>,

        /// # The active instructions
        active_instructions: Vec<InstructionAddress>,

        /// # The operands in the current stack frame
        current_operands: Vec<Value>,
    },
}
