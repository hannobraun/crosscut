use leptos::{
    component,
    prelude::{ClassAttribute, CollectView, ElementChild, IntoAny},
    view, IntoView,
};

use crate::{
    model::{ActiveFunctions, ActiveFunctionsEntry},
    ui::{
        components::{function::NamedFunction, panel::Panel},
        ActionsTx,
    },
};

#[component]
pub fn ActiveFunctions(
    active_functions: ActiveFunctions,
    actions: ActionsTx,
) -> impl IntoView {
    let active_functions = match active_functions {
        ActiveFunctions::Entries { entries } => {
            let functions = entries
                .inner
                .into_iter()
                .map(|entry| {
                    let function = match entry {
                        ActiveFunctionsEntry::Function(function) => function,
                        ActiveFunctionsEntry::Gap => {
                            return view! {
                                <span class="inline-block max-w-xl p-4 font-bold text-red-600">
                                    <p>
                                        "Functions that should be displayed \
                                        here are omitted. This is the result \
                                        of a compiler optimization."
                                    </p>
                                    <p class="mt-4">
                                        "It's possible to figure out which \
                                        functions are missing here, and still \
                                        display them correctly. But we're not \
                                        quite there yet. Sorry for the \
                                        inconvenience!"
                                    </p>
                                </span>
                            }
                            .into_any()
                        }
                    };

                    view! {
                        <NamedFunction
                            function=function
                            actions=actions.clone() />
                    }
                    .into_any()
                })
                .collect_view();

            view! {
                <ol>
                    {functions}
                </ol>
            }
            .into_any()
        }
        ActiveFunctions::Message { message } => view! {
            <p
                class="w-full h-full absolute inset-y-0 flex \
                    justify-center items-center">
                {message.to_string()}
            </p>
        }
        .into_any(),
    };

    view! {
        <Panel class="h-80">
            {active_functions}
        </Panel>
    }
}
