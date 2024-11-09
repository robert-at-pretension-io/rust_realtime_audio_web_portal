use wasm_bindgen::prelude::*;
use webrtc_vad::{Vad, SampleRate, VadMode};

#[wasm_bindgen]
pub struct WasmVad {
    vad: Vad,
}

#[wasm_bindgen]
impl WasmVad {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        WasmVad {
            vad: Vad::new()
        }
    }

    #[wasm_bindgen]
    pub fn new_with_rate_and_mode(sample_rate: u32, mode: u32) -> Result<WasmVad, JsValue> {
        let sample_rate = match sample_rate {
            8000 => SampleRate::Rate8kHz,
            16000 => SampleRate::Rate16kHz,
            32000 => SampleRate::Rate32kHz,
            48000 => SampleRate::Rate48kHz,
            _ => return Err(JsValue::from_str("Invalid sample rate")),
        };

        let mode = match mode {
            0 => VadMode::Quality,
            1 => VadMode::LowBitrate,
            2 => VadMode::Aggressive,
            3 => VadMode::VeryAggressive,
            _ => return Err(JsValue::from_str("Invalid VAD mode")),
        };

        Ok(WasmVad {
            vad: Vad::new_with_rate_and_mode(sample_rate, mode)
        })
    }

    #[wasm_bindgen]
    pub fn process_audio(&mut self, audio_data: &[i16]) -> Result<bool, JsValue> {
        self.vad.is_voice_segment(audio_data)
            .map_err(|_| JsValue::from_str("Invalid frame length"))
    }
}
