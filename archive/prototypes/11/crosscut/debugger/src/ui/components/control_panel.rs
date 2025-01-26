use leptos::{component, view, IntoView};

use crate::{
    model::UserAction,
    ui::{
        components::{button::Button, panel::Panel},
        ActionsTx,
    },
};

#[component]
pub fn ControlPanel(actions: ActionsTx) -> impl IntoView {
    view! {
        <Panel class="">
            <Button
                label="Reset"
                action=UserAction::Reset
                actions=actions.clone() />
            <Button
                label="Stop"
                action=UserAction::Stop
                actions=actions.clone() />
            <Button
                label="Continue"
                action=UserAction::Continue
                actions=actions />
        </Panel>
    }
}
