use sycamore::{
    component,
    prelude::Indexed,
    reactive::{ReadSignal, Scope},
    view,
    view::View,
    web::Html,
};

use crate::{cp, render::test_report::TestReport};

#[component]
pub fn TestRunResult<'r, G: Html>(
    cx: Scope<'r>,
    props: TestReportsProps<'r>,
) -> View<G> {
    view! { cx,
        ul {
            Indexed(
                iterable=props.test_reports,
                view=|cx, test_report| view! { cx,
                    li {
                        TestReport(test_report=test_report)
                    }
                }
            )
        }
    }
}

#[derive(sycamore::Prop)]
pub struct TestReportsProps<'r> {
    test_reports: &'r ReadSignal<Vec<cp::TestReport>>,
}
