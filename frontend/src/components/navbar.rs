use leptos::*;
use leptos_oidc::{Auth, Authenticated, LoginLink, LogoutLink};
use leptos_router::*;

#[component]
pub fn NavBar() -> impl IntoView {
    let (burger_active, set_burger_active) = create_signal(false);
    let auth = expect_context::<Auth>();

    view! {
        <nav class="navbar is-warning" role="navigation">

            <div class="navbar-brand">
                <A class="navbar-item" href=""><img src={"images/lemon-solid.png"}/></A>
                <a role="button"
                   class={ move || format!("navbar-burger {}", if burger_active.get() {"is-active"} else {""})}
                   aria-label="menu"
                   aria-expanded="false"
                   data-target="navbar"
                   on:click= move |_| set_burger_active.set(!burger_active.get())
                >
                    <span aria-hidden="true"/><span aria-hidden="true"/><span aria-hidden="true"/>
                </a>
            </div>

            <div class={ move || format!("navbar-menu {}", if burger_active.get() {"is-active"} else {""})}
                 id="navbar"
            >
                <div class="navbar-start">
                    <A class="navbar-item" href=""> Home </A>
                    <A class="navbar-item" href="studio"> Studio </A>
                </div>
                {move ||auth.access_token()}
                <div class="navbar-end">
                    <A class="navbar-item" href="settings"> Settings </A>
                    <Authenticated
                     unauthenticated= move || view!{<LoginLink class="navbar-item">Sign in</LoginLink>}
                     loading= move || view! { "..." }
                    >
                        <LogoutLink class="navbar-item">Sign Out</LogoutLink>
                    </Authenticated>
                    <LogoutLink class="navbar-item">Sign Out</LogoutLink>
                </div>
            </div>
        </nav>
    }
}
