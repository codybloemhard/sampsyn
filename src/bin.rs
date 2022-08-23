use sdl2::audio::{ AudioCallback, AudioSpecDesired };

use std::time::Duration;
use std::convert::TryInto;
use std::f32::consts::PI;
use std::io::Write;

use sampsyn::*;

pub fn main(){
    let hz = 130.81; // 130.81 = c3, 261.63 = c4, 523.25 = c5
    // let table = table_from_file_from_arg(hz);
    let table = read_wavetable_from_file("table").unwrap();
    play_table(&table, 440.0, 0.0, 48000);
    // table_write(&table, "table")
}

pub fn table_from_file_from_arg(hz: f32) -> WaveTable{
    let args: Vec<String> = std::env::args().collect();
    let file = &args[1];
    let mut reader = hound::WavReader::open(file).expect("Could not open file!");
    let mut copy = Vec::new();
    for s in reader.samples::<i16>().take(5 * 44100){
        if s.is_err() { continue; }
        let s = s.unwrap();
        copy.push(s);
    }
    create_wavetable(copy, 44100, hz)
}

pub fn table_write(table: &WaveTable, file: &str){
    let bytes = bincode::serialize(table).unwrap();
    let mut buffer = std::fs::File::create(file).unwrap();
    buffer.write_all(&bytes).unwrap();
}

pub fn play_table(table: &WaveTable, hz: f32, t: f32, sr: usize){
    let samples = wavetable_act(table, hz, t, sr as f32, sr * 9);
    play_sdl_audio_mono(samples, sr, 0.99);
}

pub fn play_table_with_state(table: &WaveTable, hz: f32, sr: usize){
    let mut state = initial_state(table, 0.0);
    let mut samples = Vec::new();
    for i in 0..sr*9{
        let t = i as f32 / sr as f32;
        let s = wavetable_act_state(table, &mut state, hz, t, sr as f32);
        samples.push(s);
    }
    play_sdl_audio_mono(samples, sr, 0.99);
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

