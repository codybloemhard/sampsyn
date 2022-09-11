use sampsyn::*;

use sdl2::audio::{ AudioCallback, AudioSpecDesired };
use clap::{ Parser, Subcommand };

use std::time::Duration;
use std::convert::TryInto;
use std::f32::consts::PI;
use std::io::Write;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args{
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands{
    Build{
        #[clap(required = true, help = "The input should be a .wav sound file.")]
        input: String,
        #[clap(required = true, help = "The file name of the sampsyn wavetable.")]
        output: String,
        #[clap(long, default_value_t = 440.0, help = "Fundamental frequency of the input file.")]
        hz: f32,
    },
    Play{
        #[clap(required = true, help = "The input should be a sampsyn wavetable.")]
        input: String,
        #[clap(long, default_value_t = 440.0, help = "Frequency of the note to play.")]
        hz: f32,
        #[clap(long, default_value_t = 48000, help = "Sample rate of the playback.")]
        sr: usize,
        #[clap(long, default_value_t = 4.0, help = "Length of the playback in seconds.")]
        len: f64,
    },
    Run{
        #[clap(required = true, help = "The input should be a .wav sound file.")]
        input: String,
        #[clap(long, default_value_t = 440.0, help = "Fundamental frequency of the input file.")]
        in_hz: f32,
        #[clap(long, default_value_t = 440.0, help = "Frequency of the note to play.")]
        out_hz: f32,
        #[clap(long, default_value_t = 48000, help = "Sample rate of the playback.")]
        sr: usize,
        #[clap(long, default_value_t = 4.0, help = "Length of the playback in seconds.")]
        len: f64,
    },
}

pub fn main(){
    let args = Args::parse();
    match args.command{
        Commands::Build{ input, output, hz } => {
            // 130.81 = c3, 261.63 = c4, 523.25 = c5
            let table = table_from_file_from_arg(&input, hz);
            table_write(&table, &output)
        },
        Commands::Play{ input, hz, sr, len } => {
            let table = read_wavetable_from_file(&input).unwrap();
            play_table(&table, hz, 0.0, sr, len);
        },
        Commands::Run{ input, in_hz, out_hz, sr, len } => {
            let table = table_from_file_from_arg(&input, in_hz);
            play_table(&table, out_hz, 0.0, sr, len);
        },
    }
}

pub fn table_from_file_from_arg(file: &str, hz: f32) -> WaveTable{
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

pub fn play_table(table: &WaveTable, hz: f32, t: f32, sr: usize, secs: f64){
    let len = (sr as f64 * secs) as usize;
    let samples = wavetable_act(table, hz, t, sr as f32, len);
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

