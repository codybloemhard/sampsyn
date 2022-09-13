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
        #[clap(long, default_value = "", help = "Note of the input file.")]
        note: String,
    },
    Play{
        #[clap(required = true, help = "The input should be a sampsyn wavetable.")]
        input: String,
        #[clap(long, default_value_t = 440.0, help = "Frequency of the note to play.")]
        hz: f32,
        #[clap(long, default_value = "", help = "Note to play.")]
        note: String,
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
        #[clap(long, default_value = "", help = "Note of the input file.")]
        in_note: String,
        #[clap(long, default_value_t = 440.0, help = "Frequency of the note to play.")]
        out_hz: f32,
        #[clap(long, default_value = "", help = "Note to play.")]
        out_note: String,
        #[clap(long, default_value_t = 48000, help = "Sample rate of the playback.")]
        sr: usize,
        #[clap(long, default_value_t = 4.0, help = "Length of the playback in seconds.")]
        len: f64,
    },
}

pub fn main(){
    let args = Args::parse();
    match args.command{
        Commands::Build{ input, output, hz, note } => {
            let hz = get_hz(&note, hz);
            let table = table_from_file_from_arg(&input, hz);
            table_write(&table, &output)
        },
        Commands::Play{ input, hz, note, sr, len } => {
            let hz = get_hz(&note, hz);
            let table = read_wavetable_from_file(&input).unwrap();
            play_table(&table, hz, 0.0, sr, len);
        },
        Commands::Run{ input, in_hz, in_note, out_hz, out_note, sr, len } => {
            let in_hz = get_hz(&in_note, in_hz);
            let out_hz = get_hz(&out_note, out_hz);
            let table = table_from_file_from_arg(&input, in_hz);
            play_table(&table, out_hz, 0.0, sr, len);
        },
    }
}

fn get_hz(note: &str, hz: f32) -> f32{
    if !note.is_empty(){
        let note = str_to_note(note)
            .unwrap_or_else(|| panic!("Argument \"{}\" could be parsed as a note value!", note));
        to_pitch(note)
    } else {
        hz
    }
}

fn to_pitch(note: usize) -> f32{
    let x = note as i32 - 48;
    (2.0f32).powf(x as f32 / 12.0) * 220.0f32
}

fn str_to_note(string: &str) -> Option<usize>{
    let chars: [char; 3] = string.chars().cycle().take(3).collect::<Vec<_>>().try_into().ok()?;
    let (offset, octave) = match chars{
        ['a', 's', n] => (1, n),
        ['a',   n, _] => (0, n),
        ['b',   n, _] => (2, n),
        ['c', 's', n] => (4, n),
        ['c',   n, _] => (3, n),
        ['d', 's', n] => (6, n),
        ['d',   n, _] => (5, n),
        ['e',   n, _] => (7, n),
        ['f', 's', n] => (9, n),
        ['f',   n, _] => (8, n),
        ['g', 's', n] => (11, n),
        ['g',   n, _] => (10, n),
        _ => return None,
    };
    octave.to_string().parse::<usize>().map(|oct| offset + oct * 12).ok()
}

fn table_from_file_from_arg(file: &str, hz: f32) -> WaveTable{
    let mut reader = hound::WavReader::open(file).expect("Could not open file!");
    let mut copy = Vec::new();
    for s in reader.samples::<i16>().take(5 * 44100){
        if s.is_err() { continue; }
        let s = s.unwrap();
        copy.push(s);
    }
    create_wavetable(copy, 44100, hz)
}

fn table_write(table: &WaveTable, file: &str){
    let bytes = bincode::serialize(table).unwrap();
    let mut buffer = std::fs::File::create(file).unwrap();
    buffer.write_all(&bytes).unwrap();
}

fn play_table(table: &WaveTable, hz: f32, t: f32, sr: usize, secs: f64){
    let len = (sr as f64 * secs) as usize;
    let samples = wavetable_act(table, hz, t, sr as f32, len);
    play_sdl_audio_mono(samples, sr, 0.99);
}

fn play_table_with_state(table: &WaveTable, hz: f32, sr: usize){
    let mut state = initial_state(table, 0.0);
    let mut samples = Vec::new();
    for i in 0..sr*9{
        let t = i as f32 / sr as f32;
        let s = wavetable_act_state(table, &mut state, hz, t, sr as f32);
        samples.push(s);
    }
    play_sdl_audio_mono(samples, sr, 0.99);
}

fn test_sdl_audio(){
    let sr = 44100;
    let secs = 2;
    let mut samples = Vec::new();
    for s in 0..secs * sr{
        samples.push((s as f32 / sr as f32 * 440f32 * 2f32 * PI).sin())
    }
    play_sdl_audio_mono(samples, sr, 1f32);
}

fn play_sdl_audio_mono(samples: Vec<f32>, sample_rate: usize, volume: f32){
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

#[cfg(test)]
mod tests{

    use super::*;

    #[test]
    fn test_str_to_note(){
        assert_eq!(str_to_note("a0"), Some(0));
        assert_eq!(str_to_note("as0"), Some(1));
        assert_eq!(str_to_note("b0"), Some(2));
        assert_eq!(str_to_note("c0"), Some(3));
        assert_eq!(str_to_note("cs0"), Some(4));
        assert_eq!(str_to_note("d0"), Some(5));
        assert_eq!(str_to_note("ds0"), Some(6));
        assert_eq!(str_to_note("e0"), Some(7));
        assert_eq!(str_to_note("f0"), Some(8));
        assert_eq!(str_to_note("fs0"), Some(9));
        assert_eq!(str_to_note("g0"), Some(10));
        assert_eq!(str_to_note("gs0"), Some(11));
        assert_eq!(str_to_note("a1"), Some(12));
        assert_eq!(str_to_note("cs1"), Some(16));
    }
}
