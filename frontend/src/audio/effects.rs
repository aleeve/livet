use gloo_console::log;
use serde_json::json;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{self, window, MediaStreamAudioSourceNode};
use web_sys::{AnalyserNode, AudioContext, AudioNode, GainNode, MediaStream};

//TODO wire upp proper effects
pub fn analyse(parent: &AudioNode, ctx: &AudioContext) -> Result<AnalyserNode, JsValue> {
    let analyser = ctx.create_analyser().unwrap();
    analyser.set_fft_size(256);
    parent.connect_with_audio_node(&analyser)?;
    Ok(analyser)
}

pub fn gain(ctx: &AudioContext) -> Result<GainNode, JsValue> {
    let gain = ctx.create_gain().unwrap();
    Ok(gain)
}
