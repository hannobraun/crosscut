use capi_runtime::Instructions;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum Command {
    ClearBreakpointAndContinue,
    ClearBreakpointAndEvaluateNextInstruction,
    Reset,
    Stop,
    UpdateCode { instructions: Instructions },
}
