use rand::prelude::*;
use std::f32::consts::PI;
pub type Fluctuation = (f32,f32); // sin period(sec), sin amplitude
// property of harmonic: amplitude, duration to ampl = 0(sec)
pub type SimpleHarmF = (f32,f32);
pub type SimpleInstrument = Vec<SimpleHarmF>;
// property of harmonic: amplitude, duration to ampl = 0(sec), amplitude fluc, freq fluc
pub type HarmF = (f32,f32,Fluctuation,Fluctuation);
pub type Instrument = Vec<HarmF>;

pub fn simple_tone(hz: f32, sr: usize, len: f32, inst: SimpleInstrument) -> Vec<f32>{
    let mut samples = Vec::new();
    for s in 0..(len * sr as f32) as usize{
        let t = s as f32 / sr as f32;
        let mut s = 0.0;
        let mut amp = 0.0;
        for (i,(a,l)) in inst.iter().enumerate(){
            let i = i + 1;
            let ohz = (hz * i as f32).min(20000.0);
            let a = a.max(0.0);
            s += sine_sample(t, ohz) * a * ((l*len - t) / len).max(0.0);
            amp += a;
        }
        s /= amp;
        samples.push(s);
    }
    samples
}

pub fn tone(hz: f32, sr: usize, len: f32, inst: Instrument) -> Vec<f32>{
    let mut samples = Vec::new();
    for s in 0..(len * sr as f32) as usize{
        let t = s as f32 / sr as f32;
        let mut s = 0.0;
        let mut amp = 0.0;
        for (i,(a,l,af,ff)) in inst.iter().enumerate(){
            let i = i + 1;
            let ohz = hz * i as f32;
            if ohz > 20000.0 || a < &0.001 { break; }
            s += sine_sample(t, ohz + fluctuate(t, *ff)) * (a + fluctuate(t, *af)) * ((l*len - t) / len).max(0.0);
            amp += a;
        }
        s /= amp;
        samples.push(s);
    }
    samples
}

pub fn sine_sample(t: f32, hz: f32) -> f32{
    (t * hz * 2f32 * PI).sin()
}

pub fn fluctuate(t: f32, (p,a): Fluctuation) -> f32{
    (t * 2f32 * PI / p).sin() * a
}

pub fn learn_simple_instrument(sample: &[i16], sr: usize, hz: f32, hs: usize) -> SimpleInstrument{
    let hs = hs.min((20000.0 / hz) as usize);
    let hsf = hs as f32;
    let len = sample.len() as f32 / sr as f32;
    let mut instr = vec![(1. / hsf, len); hs];
    fn cerror(og: &[i16], inp: &[f32]) -> usize{
        let mut error = 0;
        for i in 0..og.len().min(inp.len()){
            let temp = og[i] as i64 - (inp[i] * std::i16::MAX as f32) as i64;
            error += (temp * temp) as usize;
        }
        error
    }
    macro_rules! wchoose{
        ( $rng:expr, $chances:expr, $( $x:block ),+ ) => {
            {
                let r = $rng.gen::<f32>() * $chances.iter().fold(0.0, |s,v| s + v);
                let mut c = 0.0;
                let mut j = 0;
                for (i, chance) in $chances.iter().enumerate(){
                    c += chance;
                    if r < c {
                        j = i;
                        break;
                    }
                };
                let mut i = 0;
                let mut res = Option::None;
                $(
                    i += 1;
                    if i - 1 == j { res = Option::Some($x); }
                )*
                res
            }
        };
    }
    pub fn choose(rng: &mut ThreadRng, len: usize) -> usize{
        rng.gen::<usize>() % len
    }
    pub fn mut_amp(harm: SimpleHarmF, delta: f32) -> SimpleHarmF{
        (harm.0 + delta, harm.1)
    }
    pub fn mut_len(harm: SimpleHarmF, delta: f32) -> SimpleHarmF{
        (harm.0, harm.1 + delta)
    }
    let mut error = std::usize::MAX;
    let mut rng = rand::thread_rng();
    let mut i = 0;
    let mut sens = 0.2;
    loop{
        let old_error = error;
        for _ in 0..100{
            let h = choose(&mut rng, hs);
            let mut inst_clone = instr.clone();
            wchoose!(&mut rng, &[0.3, 0.3, 0.2, 0.2],
                {inst_clone[h] = mut_amp(inst_clone[h], sens)},
                {inst_clone[h] = mut_amp(inst_clone[h], -sens)},
                {inst_clone[h] = mut_len(inst_clone[h], sens)},
                {inst_clone[h] = mut_len(inst_clone[h], -sens)}
            );
            let tone = simple_tone(hz, sr, len, inst_clone.clone());
            let err = cerror(&sample, &tone);
            if err < error{
                error = err;
                instr = inst_clone;
            }
            i += 1;
        }
        println!("---- {}, ", error);
        if old_error == error{
            sens *= 0.5;
            println!("-- sens: {}, ", sens);
            if sens < 0.001 { break; }
        }
    }
    println!("Did {} iterations!", i);
    instr
}

// some different shit idk what's this is turning into

// (hz, sr, samples, [(time, wave)])
pub type WaveTable = (f32, usize, usize, Vec<(f32, Vec<f32>)>);

pub fn into_mono(stereo: Vec<i16>) -> Vec<f32>{
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
    (hz, sr, samples, waves)
}

pub fn wavetable_act((waves_hz, sr, samples, waves): &WaveTable, hz: f32, t: f32, len: usize) -> Vec<f32>{
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
    let off = (t * *sr as f32) as usize;
    for i in 0..len{
        let tt = t + (i as f32 / *sr as f32);
        if tt > waves[b].0 && b < waves.len() - 1{
            a += 1;
            b += 1;
            diff = waves[b].0 - waves[a].0;
        }
        // lerp between frames
        let float_frame = (off + i) as f32 * (hz / waves_hz);
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
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

// pub fn write_test_tone(){
//     let spec = hound::WavSpec{
//         channels: 1,
//         sample_rate: 44100,
//         bits_per_sample: 16,
//         sample_format: hound::SampleFormat::Int,
//     };
//     let mut writer = hound::WavWriter::create("sine.wav", spec).unwrap();
//     for t in (0..44100).map(|x| x as f32 / 44100.0){
//         let sample = (t * 440.0 * 2.0 * PI).sin();
//         let ampl = i16::MAX as f32;
//         writer.write_sample((sample * ampl) as i16).unwrap();
//     }
// }
//
// pub fn write_test_tone_stereo(){
//     let spec = hound::WavSpec{
//         channels: 2,
//         sample_rate: 44100,
//         bits_per_sample: 16,
//         sample_format: hound::SampleFormat::Int,
//     };
//     let mut writer = hound::WavWriter::create("sine.wav", spec).unwrap();
//     for t in (0..44100).map(|x| x as f32 / 44100.0){
//         let sa = (t * 220.0 * 2.0 * PI).sin();
//         let sb = (t * 440.0 * 2.0 * PI).sin();
//         let ampl = i16::MAX as f32;
//         writer.write_sample((sa * ampl) as i16).unwrap();
//         writer.write_sample((sb * ampl) as i16).unwrap();
//     }
// }
