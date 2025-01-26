use leptos::{
    children::Children,
    component,
    prelude::{ClassAttribute, ElementChild},
    view, IntoView,
};

#[component]
pub fn Panel(children: Children, class: &'static str) -> impl IntoView {
    let class =
        format!("mx-1 my-3 border p-1 relative overflow-y-auto {class}");

    view! {
        <div class=class>
            {children()}
        </div>
    }
}
