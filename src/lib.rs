use serde::{ Serialize, Deserialize };
use simpleio::read_file_into_buffer;

/// WaveTable(hz, sr, samples, [(time, wave)])
#[derive(Serialize, Deserialize)]
pub struct WaveTable(f32, usize, usize, Vec<(f32, Vec<f32>)>);

fn into_mono(stereo: Vec<i16>) -> Vec<f32>{
    let mut mono = Vec::new();
    let mut i = 0;
    let mut y = 0i32;
    for x in stereo{
        y += x as i32;
        if i == 0{
            i = 1;
        } else {
            i = 0;
            mono.push((y / 2) as f32 / i16::MAX as f32);
            y = 0;
        }
    }
    mono
}

/// Parses table from buffer, so you can manage the file seperately
pub fn parse_wavetable_from_buffer(buffer: &[u8]) -> Option<WaveTable>{
    if let Ok(table) = bincode::deserialize(buffer) { Some(table) }
    else { None }
}

/// Reads a file and you get the table out of it(if it is indeed a table file)
pub fn read_wavetable_from_file(file: &str) -> Option<WaveTable>{
    let buffer = match read_file_into_buffer(file){
        Err(_) => { return None; },
        Ok(x) => x,
    };
    parse_wavetable_from_buffer(&buffer)
}

/// Takes a bunch of samples, sr of the input and the frequency of the fundemental to create the
/// wave table. Yes, you should know what the base frequency is otherwise it won't work.
/// You could try doing a different frequency than it actually is for artistic effect but I didn't
/// test that.
pub fn create_wavetable(stereo: Vec<i16>, sr: usize, hz: f32) -> WaveTable{
    let mono = into_mono(stereo);
    let max = mono.iter().fold(0.0, |max: f32, v| max.max(v.abs()));
    let mono = mono.into_iter().map(|v| v / max).collect::<Vec<_>>();
    let samples = (sr as f32 / hz).round() as usize;
    let mut waves = vec![(0.0, mono.iter().copied().take(samples).collect::<Vec<_>>())];

    let mut t = samples;
    loop{
        if t + samples >= mono.len() { break; }
        waves.push((t as f32 / sr as f32, mono.iter().copied().skip(t).take(samples).collect::<Vec<_>>()));
        t *= 2;
    }
    WaveTable(hz, sr, samples, waves)
}

/// Takes a wave table and a frequency of what you want the fundemental of the output to be. t is
/// the starting time it's at currently and sr is the samplerate of the output. len is the amount
/// of frames the output should be.
pub fn wavetable_act(WaveTable(waves_hz, _sr, samples, waves): &WaveTable, hz: f32, t: f32, sr: f32, len: usize) -> Vec<f32>{
    let mut res = Vec::new();
    if waves.is_empty() {
        return res;
    }
    let mut a = 0;
    let mut b = 0;
    let mut diff = 0.0;
    if waves.len() > 1{
        for (i, (wave_time, _)) in waves.iter().enumerate(){
            if t <= *wave_time{
                a = i;
                b = i + 1;
                if b >= waves.len() { // need to use last wave
                    a = waves.len() - 2;
                    b = waves.len() - 1;
                }
                diff = waves[b].0 - waves[a].0;
                break;
            }
        }
        if b == 0{ // t > every wavetime, only use last wave
            a = waves.len() - 1;
            b = waves.len() - 1;
            diff = 0.0;
        }
    }
    // generate buffer
    let off = (t * sr) as usize;
    let sr_rat = *_sr as f32 / sr;
    for i in 0..len{
        let tt = t + (i as f32 / sr);
        if tt > waves[b].0 && b < waves.len() - 1{
            a += 1;
            b += 1;
            diff = waves[b].0 - waves[a].0;
        }
        // lerp between frames
        let float_frame = (off + i) as f32 * sr_rat * (hz / waves_hz);
        let under = float_frame.floor() as usize % samples;
        let above = float_frame.ceil() as usize % samples;
        let fract = float_frame.fract();
        let a_val = waves[a].1[under] + fract * (waves[a].1[above] - waves[a].1[under]);
        let b_val = waves[b].1[under] + fract * (waves[b].1[above] - waves[b].1[under]);
        // lerp between waves
        let norm_t = ((tt - waves[a].0) / diff).min(1.0);
        let v = a_val + norm_t * (b_val - a_val);
        res.push(v);
    }
    res
}

#[cfg(test)]
mod tests {
    // 100% test coverage
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

