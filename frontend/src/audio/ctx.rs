use gloo_console::log;
use leptos::*;
use serde_json::json;
use uuid::Uuid;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    window, AnalyserNode, AudioContext, GainNode, MediaStream, MediaStreamAudioSourceNode,
    MediaStreamTrack,
};

use super::{analyse, gain};

#[derive(Clone, Debug)]
pub struct AudioTrack {
    pub owner: Uuid,
    pub label: String,
    stream: MediaStream,
    source: MediaStreamAudioSourceNode,
    pub gain: GainNode,
    pub analyser: AnalyserNode,
}
impl AudioTrack {
    pub fn new(
        ctx: &AudioContext,
        owner: Uuid,
        track: &MediaStreamTrack,
    ) -> Result<AudioTrack, JsValue> {
        let stream = MediaStream::new()?;
        stream.add_track(track);
        let source = ctx.create_media_stream_source(&stream)?;
        let gain = ctx.create_gain()?;
        let analyser = ctx.create_analyser()?;

        source
            .connect_with_audio_node(&gain)?
            .connect_with_audio_node(&analyser)?;

        let track = AudioTrack {
            owner,
            label: track.label(),
            stream,
            source,
            gain,
            analyser,
        };
        Ok(track)
    }

    pub fn get_track(&self) -> Result<MediaStreamTrack, JsValue> {
        let track = self.stream.get_audio_tracks().get(0);
        Ok(MediaStreamTrack::from(track))
    }
}

#[derive(Clone, Debug)]
pub struct AudioGraph {
    ctx: AudioContext,
    owner: Uuid,
    pub local_tracks: Vec<AudioTrack>,
    pub remote_tracks: Vec<AudioTrack>,
    pub gain: GainNode,
    pub analyser: AnalyserNode,
}

impl AudioGraph {
    pub fn new() -> Result<AudioGraph, JsValue> {
        let ctx = AudioContext::new()?;
        let gain = gain(&ctx)?;
        let analyser = analyse(&gain, &ctx)?;

        let graph = Self {
            owner: Uuid::new_v4(),
            ctx,
            gain,
            analyser,
            local_tracks: vec![],
            remote_tracks: vec![],
        };
        Ok(graph)
    }

    pub fn connect(&mut self) -> Result<(), JsValue> {
        let destination = self.ctx.destination();
        self.gain.connect_with_audio_node(&destination)?;
        Ok(())
    }

    pub fn suspend(&mut self) -> Result<(), JsValue> {
        let fut = JsFuture::from(self.ctx.suspend()?);
        spawn_local(async {
            log!("Suspend");
            fut.await.unwrap();
        });
        Ok(())
    }

    pub fn resume(&mut self) -> Result<(), JsValue> {
        let fut = JsFuture::from(self.ctx.resume()?);
        spawn_local(async {
            log!("Resume");
            fut.await.unwrap();
        });
        Ok(())
    }

    pub async fn add_device(&mut self, device: &str) -> Result<(), JsValue> {
        let audio: String = json!(
        {
            "latencyHint": 0.000001,
            "latency": 0,
            "echoCancellation": false,
            "noiseSuppression": false,
            "autoGainControl": false,
            "deviceId":{
                "exact": device
            }
        })
        .to_string();

        let mut constraints = web_sys::MediaStreamConstraints::new();
        constraints.audio(&JsValue::from(&audio));

        let devices = window().expect("Oh my god").navigator().media_devices()?;

        let stream = devices.get_user_media_with_constraints(&constraints)?;
        let stream = JsFuture::from(stream).await?;
        let stream = MediaStream::from(stream);

        let tracks: Vec<MediaStreamTrack> = stream
            .get_audio_tracks()
            .iter()
            .map(|j_track| MediaStreamTrack::from(j_track))
            .collect();
        for track in &tracks {
            let track = AudioTrack::new(&self.ctx, self.owner, track)?;
            track.gain.connect_with_audio_node(&self.gain)?;
            self.local_tracks.push(track);
        }
        Ok(())
    }

    pub fn add_input(&mut self, owner: Uuid, track: MediaStreamTrack) -> Result<(), JsValue> {
        let track = AudioTrack::new(&self.ctx, owner, &track)?;
        track.gain.connect_with_audio_node(&self.gain)?;
        self.remote_tracks.push(track);
        Ok(())
    }

    pub fn stream(&self) -> Result<MediaStream, JsValue> {
        let out_stream = self.ctx.create_media_stream_destination().unwrap().stream();
        for track in &self.local_tracks {
            out_stream.add_track(&track.get_track()?);
        }
        Ok(out_stream)
    }
}
