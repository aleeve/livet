use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::prelude::*;
use web_sys::{self, window};
use gloo_utils::format::JsValueSerdeExt;
use serde::{Serialize,Deserialize};

pub async fn get_devices() -> Result<Vec<InputDeviceInfo>, JsValue> {
    let mut constraints = web_sys::MediaStreamConstraints::new();
    constraints.audio(&JsValue::from(true));

    let devices = window().expect("Oh my god")
        .navigator()
        .media_devices().unwrap();

    let enumeration = devices.enumerate_devices().unwrap();
    let devices = JsFuture::from(enumeration)
        .await.unwrap().into_serde::<Vec<InputDeviceInfo>>().unwrap();
    Ok(devices)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InputDeviceInfo {
    pub deviceId:String,
    pub groupId:String,
    pub kind:String,
    pub label:String,
}
