use sdl2::audio::{AudioCallback, AudioSpecDesired};
use std::time::Duration;
use std::convert::TryInto;
use std::f32::consts::PI;
use otsyn;

pub fn main(){
    let sr = 44100;
    let instrument = vec![(1.0,2.0),(1.1,1.0),(1.3,1.5),(1.5,1.0),(1.3,1.0),(1.1,1.0),(0.9,1.0),(0.7,1.0),(0.5,1.0),(0.3,1.0),(0.1,1.0)];
    let samples = otsyn::tone(220.0,sr, 2.0, instrument);
    play_sdl_audio_mono(samples, sr, 0.9);
}

pub fn test_sdl_audio(){
    let sr = 44100;
    let secs = 2;
    let mut samples = Vec::new();
    for s in 0..secs * sr{
        samples.push((s as f32 / sr as f32 * 440f32 * 2f32 * PI).sin())
    }
    play_sdl_audio_mono(samples, sr, 1f32);
}

pub fn play_sdl_audio_mono(samples: Vec<f32>, sample_rate: usize, volume: f32){
    struct Sound{
        index: usize,
        samples: Vec<f32>,
        volume: f32,
    }

    impl AudioCallback for Sound{
        type Channel = f32;

        fn callback(&mut self, out: &mut [f32]){
            for x in out.iter_mut(){
                *x = if self.index < self.samples.len(){
                    self.samples[self.index] * self.volume
                }else{
                    0f32
                };
                self.index += 1;
            }
        }
    }

    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(sample_rate.try_into().unwrap()),
        channels: Some(1),
        samples: None
    };

    let slen = samples.len();
    let device = audio_subsystem.open_playback(None, &desired_spec, |_spec| {
        // initialize the audio callback
        Sound{
            index: 0,
            samples,
            volume,
        }
    }).unwrap();

    device.resume();
    std::thread::sleep(Duration::from_millis((slen / sample_rate * 1000).try_into().unwrap()));
}

