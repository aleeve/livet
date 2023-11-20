use leptos::*;
use std::time::Duration;
use web_sys::AnalyserNode;

#[component]
pub fn LevelMeter(analyser: AnalyserNode) -> impl IntoView {
    let (value, set_value) = create_signal(0.0);
    let value = move || value.get().to_string();

    let calculate_colume = move || {
        let mut array = [0u8; 64];
        analyser.get_byte_frequency_data(&mut array);
        let power = array
            .iter()
            .map(|v| (f32::from(v.clone()).powi(2)))
            .sum::<f32>();
        let volume = (power / 64.0f32).sqrt();
        set_value.set(volume);
    };

    set_interval(calculate_colume, Duration::from_millis(100));

    view! {<aux-levelmeter
      min="0"
      max="100"
      segment="4"
      value=value
      id="meter"
      show_hold="true"
      label="Out"
      show_clip= "true"
      clipping = "80"
      auto_clip = "100"
      style="height: 150px;"
      layout="right"
      background="#2577a1"
      foreground="#e0e0e0"
      auto_hold="100"
      show_hold="true"
      falling="50"
    />}
}
