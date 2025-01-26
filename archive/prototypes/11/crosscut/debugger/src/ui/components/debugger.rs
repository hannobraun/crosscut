use leptos::{
    component,
    prelude::{ElementChild, Get, ReadSignal},
    view, IntoView,
};

use crate::{
    model::{PersistentState, TransientState},
    ui::{
        components::{
            active_functions::ActiveFunctions, control_panel::ControlPanel,
            memory_explorer::MemoryExplorer, stack_explorer::StackExplorer,
        },
        ActionsTx,
    },
};

#[component]
pub fn Debugger(
    state: ReadSignal<(PersistentState, TransientState)>,
    actions: ActionsTx,
) -> impl IntoView {
    move || {
        let (persistent, transient) = state.get();

        let stack_explorer = view! {
            <StackExplorer
                current=transient.operands />
        };
        let memory_explorer = persistent.memory.map(|memory| {
            view! {
                <MemoryExplorer
                    memory=memory />
            }
        });

        view! {
            <div>
                <ControlPanel
                    actions=actions.clone() />
                <ActiveFunctions
                    active_functions=transient.active_functions
                    actions=actions.clone() />
                {stack_explorer}
                {memory_explorer}
            </div>
        }
    }
}
