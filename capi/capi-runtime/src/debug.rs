use crate::{Function, Functions};

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct DebugState {
    pub functions: Vec<Function>,
}

impl DebugState {
    pub fn new(functions: Functions) -> Self {
        let functions = functions.inner.into_iter().map(Into::into).collect();
        Self { functions }
    }

    pub fn apply_event(&mut self, event: DebugEvent) {
        match event {
            DebugEvent::ToggleBreakpoint {
                location: LineLocation { function, line },
            } => {
                let line: usize = line.try_into().unwrap();

                let function = self
                    .functions
                    .iter_mut()
                    .find(|f| f.name == function)
                    .unwrap();
                let syntax_element = function.syntax.get_mut(line).unwrap();

                syntax_element.breakpoint = !syntax_element.breakpoint;
            }
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum DebugEvent {
    ToggleBreakpoint { location: LineLocation },
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct LineLocation {
    pub function: String,
    pub line: u32,
}
