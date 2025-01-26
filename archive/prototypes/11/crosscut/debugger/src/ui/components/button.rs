use leptos::{
    component,
    prelude::{ClassAttribute, OnAttribute},
    view, IntoView,
};

use crate::{
    model::UserAction,
    ui::{actions::send_action, ActionsTx},
};

#[component]
pub fn Button(
    label: &'static str,
    action: UserAction,
    actions: ActionsTx,
) -> impl IntoView {
    let on_click = move |_| {
        leptos::task::spawn_local(send_action(action.clone(), actions.clone()));
    };

    view! {
        <input
            type="button"
            value=label
            class="m-1 px-1 bg-gray-300 font-bold"
            on:click=on_click />
    }
}
