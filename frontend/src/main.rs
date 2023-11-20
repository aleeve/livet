mod audio;
mod components;
mod network;
mod pages;

use crate::components::{Auth, NavBar};
use crate::pages::{Home, Settings, Studio};
use console_error_panic_hook;
use leptos::*;
use leptos_router::*;
use std::panic;

fn main() {
    // Add nice console stacktraces
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    // Mount the actual app
    mount_to_body(|| view! { <App/>})
}

#[component]
fn App() -> impl IntoView {
    view! {
    <div id="root">
        <Router>
            <Auth>
                <NavBar/>
                <main>
                    <Routes>
                        <Route
                            path=""
                            view=Home
                        />
                        <Route
                            path="settings"
                            view=Settings
                        />
                        <Route
                            path="studio"
                            view=Studio
                        />
                    </Routes>
                </main>
            </Auth>
        </Router>
    </div>
    }
}
