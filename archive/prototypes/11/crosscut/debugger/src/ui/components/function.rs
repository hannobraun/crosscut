use std::fmt::Write;

use crosscut_runtime::Effect;
use leptos::{
    component,
    ev::MouseEvent,
    prelude::{
        AnyView, ClassAttribute, CollectView, CustomAttribute, ElementChild,
        IntoAny, OnAttribute,
    },
    view,
    wasm_bindgen::JsCast,
    web_sys::HtmlSpanElement,
    IntoView,
};

use crate::{
    model::{
        DebugFunction, DebugMember, DebugMemberData, DebugMemberKind,
        DebugNamedFunction, DebugParameter, UserAction,
    },
    ui::{actions::send_action, ActionsTx},
};

use super::button::Button;

#[component]
pub fn NamedFunction(
    function: DebugNamedFunction,
    actions: ActionsTx,
) -> impl IntoView {
    view! {
        <div class="m-2 mb-4">
            <span class="mr-2 font-bold">
                {function.name}:
            </span>
            <Function
                function=function.inner
                actions=actions />
        </div>
    }
}

#[component]
fn Function(function: DebugFunction, actions: ActionsTx) -> impl IntoView {
    let branches = function
        .branches
        .into_iter()
        .map(|branch| {
            view! {
                <Branch
                    parameters=branch.parameters
                    body=branch.body
                    actions=actions.clone() />
            }
        })
        .collect::<Vec<_>>();

    let signature = function
        .signature
        .map(|signature| format!(": {signature}"))
        .unwrap_or_default();

    view! {
        <span>
            "fn"
            <ol>
                {branches}
            </ol>
            "end"{signature}
        </span>
    }
}

#[component]
fn Branch(
    parameters: Vec<DebugParameter>,
    body: Vec<DebugMember>,
    actions: ActionsTx,
) -> impl IntoView {
    let parameters = parameters
        .into_iter()
        .map(|DebugParameter { name, type_ }| {
            let type_ = type_
                .map(|type_| format!(": {type_}"))
                .unwrap_or(String::new());
            format!("{name}{type_}")
        })
        .collect::<Vec<_>>()
        .join(", ");
    let members = body
        .into_iter()
        .map(|member| {
            view! {
                <li class="ml-8">
                    <Member
                        member=member
                        actions=actions.clone() />
                </li>
            }
        })
        .collect_view();

    view! {
        <div class="pl-8">
            "\\" <span class="mx-2">{parameters}</span> "->"
            <ol>
                {members}
            </ol>
        </div>
    }
}

#[component]
pub fn Member(member: DebugMember, actions: ActionsTx) -> impl IntoView {
    let mut class_outer = String::from("py-1");

    let (expression, actions, error) = match member.kind {
        DebugMemberKind::Comment { lines } => {
            let lines = lines
                .into_iter()
                .map(|line| {
                    view! {
                        <p># {line}</p>
                    }
                })
                .collect::<Vec<_>>();

            let class_inner = String::from("italic text-gray-500");
            (
                view! {
                    <div class=class_inner>
                        {lines}
                    </div>
                }
                .into_any(),
                None,
                None,
            )
        }
        DebugMemberKind::Function { function } => (
            view! {
                <Function
                    function=function
                    actions=actions />
            }
            .into_any(),
            None,
            None,
        ),
        DebugMemberKind::Identifier { name } => {
            make_single_member(name, member.data, &mut class_outer, actions)
        }
        DebugMemberKind::Value { as_string } => make_single_member(
            as_string,
            member.data,
            &mut class_outer,
            actions,
        ),
    };

    view! {
        <span>
            <span class=class_outer>
                {expression}
            </span>
            {actions}
            {error}
        </span>
    }
}

fn make_single_member(
    expression: String,
    data: DebugMemberData,
    class_outer: &mut String,
    actions_tx: ActionsTx,
) -> (AnyView, Option<AnyView>, Option<AnyView>) {
    if data.has_durable_breakpoint {
        class_outer.push_str(" bg-blue-300");
    }

    let mut class_inner = String::from("px-0.5");
    if let Some(effect) = &data.effect {
        match effect {
            Effect::Breakpoint => class_inner.push_str(" bg-green-300"),
            _ => class_inner.push_str(" bg-red-300"),
        }
    }
    if data.state.is_active() {
        class_inner.push_str(" font-bold");
    }

    let data_expression = ron::to_string(&data.location).expect(
        "Expecting serialization of `ExpressionLocation` to always work.",
    );
    let data_breakpoint = data.has_durable_breakpoint;

    let actions = if data.state.is_innermost_active_expression() {
        Some(
            view! {
                <span>
                    <Button
                        label="Step In"
                        action=UserAction::StepIn
                        actions=actions_tx.clone() />
                    <Button
                        label="Step Out"
                        action=UserAction::StepOut
                        actions=actions_tx.clone() />
                    <Button
                        label="Step Over"
                        action=UserAction::StepOver
                        actions=actions_tx.clone() />
                </span>
            }
            .into_any(),
        )
    } else {
        None
    };

    let error = data.effect.map(|effect| format!("{:?}", effect));

    let toggle_breakpoint = move |event: MouseEvent| {
        let event_target = event.target().unwrap();
        let element = event_target.dyn_ref::<HtmlSpanElement>().unwrap();

        let expression = {
            let Some(expression) = element.get_attribute("data-expression")
            else {
                // This happens, if the user clicks on a comment.
                return;
            };

            ron::from_str(&expression).expect(
                "Expecting serialized expression locations in DOM to always be \
                valid.",
            )
        };

        let action = if element.has_attribute("data-breakpoint") {
            UserAction::BreakpointClear { expression }
        } else {
            UserAction::BreakpointSet { expression }
        };

        leptos::task::spawn_local(send_action(action, actions_tx.clone()));
    };

    let mut typed_expression = expression;
    if let Some(signature) = data.signature {
        write!(typed_expression, ": {signature} .")
            .expect("Writing to `String` can't fail.");
    }

    (
        view! {
            <span
                class=class_inner
                data-expression=data_expression
                data-breakpoint=data_breakpoint
                on:click=toggle_breakpoint>
                {typed_expression}
            </span>
        }
        .into_any(),
        actions,
        Some(
            view! {
                <span class="mx-2 font-bold text-red-800">
                    {error}
                </span>
            }
            .into_any(),
        ),
    )
}
