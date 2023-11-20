use leptos::*;

#[component]
pub fn Gain(value: ReadSignal<f32>) -> impl IntoView {
    view! {
        <aux-knob
          min="0"
          max="100"
          value=value
          preset="small"
          markers="js:[{from:75,to:100}]"
          labels="js:[0,25,50,75,100]"
          dots="js:[0,25,50,75,100]"
        />
    }
}

#[component]
pub fn Pan(value: ReadSignal<f32>) -> impl IntoView {
    view! {
        <aux-knob
          min="-1"
          max="1"
          value=value
          preset="small"
        />
    }
}
