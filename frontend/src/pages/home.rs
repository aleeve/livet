use gloo_console::log;
use leptos::*;

#[component]
pub fn Home() -> impl IntoView {
    let (thingy, _) = create_signal("Hi".to_string());
    view! {
        <SubHome thingy={thingy} />
    }
}

#[component]
pub fn SubHome(thingy: ReadSignal<String>) -> impl IntoView {
    create_resource(
        move || thingy.get(),
        move |msg| {
            log!("hello");
            async move {
                log!("hi!");
            }
        }
    );
    view! {
        <div>Home</div>
    }
}
