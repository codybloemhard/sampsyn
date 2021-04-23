use sdl2::audio::{AudioCallback, AudioSpecDesired};
use std::time::Duration;
use std::convert::TryInto;
use std::f32::consts::PI;
use otsyn::*;

pub fn main(){
    let sr = 44100;
    let af = (1.0,0.02);
    let ff = (1.5,2.0);
    let guitar = vec![
        (1.0,1.0,af,ff),
        (1.0,1.0,af,ff),
        (0.8,1.0,af,ff),
        (0.5,1.0,af,ff),
        (0.3,1.0,af,ff),
        (0.4,1.0,af,ff),
        (0.5,1.0,af,ff),
        (0.5,1.0,af,ff),
        (0.5,1.0,af,ff),
        (0.3,1.0,af,ff),
        (0.4,1.0,af,ff),
        (0.2,1.0,af,ff),
        (0.5,1.0,af,ff),
        (0.4,1.0,af,ff),
        (0.2,0.0,af,ff)];
    let samples = otsyn::tone(220.0,sr, 2.0, guitar);
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

// fn _test1(){
//     let sr = 44100;
//     let mut track = music_gen::tones::Track::new(sr, 2);
//     let tonef = &spread(6, 1.003, 0.0, sine_sample);
//     let volf = &hit_lin_quot_quad(40.0,0.2, 1.0, 2);
//     let hzf = &arg_id;
//     let passf = &smooth_pass(10.0);
//
//     let mut score = Score::new();
//     score.new_staff();
//     score.new_bar(0, Bar::new(Key::std_key(), 120.0, TimeSig::new(1, 1.0)));
//     score.add_note(barnote(NamedNote::A(4).to_note(), 1.0), false, 0);
//     score.add_note(barnote(NamedNote::A(4).to_note(), 1.0), false, 0);
//     score.add_note(barnote(NamedNote::Cs(4).to_note(), 1.0), true, 0);
//     score.add_note(barnote(NamedNote::E(4).to_note(), 1.0), true, 0);
//
//     println!("{}", score.as_string(0));
//
//     score.render_to_track_stereo(0, &mut track, 3.0, 1.0, 0.0, tonef, volf, hzf, passf);
//     track.trim_end(0.001);
//     track.normalize(0.99);
//     track.render("test.wav");
// }
//
// fn _test0(){
//     //let scale = ionian_mode(NamedNote::A(4).to_note(), AEOLIAN);
//     let scale = miscellaneous_scales::satie_scale_steps().as_scale(NamedNote::A(3).to_note());
//     print_notes(&scale.0, "\t");
//     let sr = 44100;
//     let mut track = music_gen::tones::Track::new(sr, 2);
//     let volf = &hit_lin_quot_quad(40.0,0.2, 1.0, 2);
//     let mut time = 0;
//     for note in scale.0{//TODO: make iter for Scale
//         let hz = to_pitch(note);
//         //tone_to_track(&mut track, time, sr * 3, 1.0, 0.0, 0.0, hz, &sine_sample, &arg_id, volf, &arg_id);
//         tone_to_track_stereo(&mut track, time, sr * 3 as usize, 1.0, 0.0, hz, &spread(6, 1.003, 0.0, sine_sample), volf, &arg_id, &smooth_pass(10.0));
//         time += sr/2;
//     }
//     track.trim_end(0.001);
//     track.normalize(0.99);
//     track.render("test.wav");
// }
