use leptos::*;

#[component]
fn Fader() -> impl IntoView {
    view! {
        <aux-fader
          min="-96"
          max="24"
          base="0"
          value="0"
          layout="right"
          style="height: 300px;"
          show_value="false"
          label="Level"
          value.size="3"
          value.format="js:function(v){return parseInt(v)+'dB'}"
        ></aux-fader>
    }
}
