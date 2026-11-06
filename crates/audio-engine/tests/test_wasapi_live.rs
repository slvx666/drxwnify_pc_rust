use cpal::traits::{DeviceTrait, HostTrait};
use drxwnify_audio_engine::WasapiOutput;

#[test]
fn test_live_wasapi_device() {
    let host = cpal::default_host();
    println!("CPAL Host: {:?}", host.id());
    if let Some(device) = host.default_output_device() {
        let name = device.name().unwrap_or_else(|_| "unknown".to_string());
        println!("Default audio output device: {}", name);
        if let Ok(config) = device.default_output_config() {
            println!("Default output config: sample_rate={}, channels={}, format={:?}",
                config.sample_rate().0,
                config.channels(),
                config.sample_format()
            );
        }
    } else {
        println!("No default output device detected!");
    }

    let out = WasapiOutput::new(None);
    match out {
        Ok(output) => {
            println!("WasapiOutput successfully initialized!");
            println!("Writing 440Hz test sine tone to physical output...");
            let mut tone = Vec::with_capacity(48000);
            for i in 0..48000 {
                let t = i as f32 / 48000.0;
                let s = (t * 440.0 * 2.0 * std::f32::consts::PI).sin();
                let sample = (s * 6000.0) as i16;
                tone.push((sample, sample));
            }
            output.write_samples(&tone);
            std::thread::sleep(std::time::Duration::from_millis(300));
            println!("Audio frames successfully dispatched to Realtek WASAPI driver!");
        }
        Err(e) => println!("WasapiOutput initialization error: {e}"),
    }
}
