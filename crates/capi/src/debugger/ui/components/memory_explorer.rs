use leptos::{component, view, CollectView, IntoView};

use crate::{debugger::ui::components::panel::Panel, state::Memory};

#[component]
pub fn MemoryExplorer(memory: Memory) -> impl IntoView {
    let mut values = memory.inner.into_iter().peekable();
    let values = values.by_ref();

    let mut lines = Vec::new();

    while values.peek().is_some() {
        let line = values.take(16).collect::<Vec<_>>();
        lines.push(line);
    }

    let lines = lines
        .into_iter()
        .map(|line| {
            view! {
                <Line line=line />
            }
        })
        .collect_view();

    view! {
        <Panel class="">
            <p>"Memory:"</p>
            <ol>
                {lines}
            </ol>
        </Panel>
    }
}

#[component]
fn Line(line: Vec<capi_process::runtime::Value>) -> impl IntoView {
    let values = line
        .into_iter()
        .map(|value| {
            view! {
                <Value value=value />
            }
        })
        .collect_view();

    view! {
        <li>
            <ol>{values}</ol>
        </li>
    }
}

#[component]
fn Value(value: capi_process::runtime::Value) -> impl IntoView {
    view! {
        <li class="inline-block w-6 mr-2 text-right">{value.to_string()}</li>
    }
}
