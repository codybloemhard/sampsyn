use rand::prelude::*;
use std::f32::consts::PI;
pub type Fluctuation = (f32,f32); // sin period(sec), sin amplitude
// property of harmonic: amplitude, duration to ampl = 0(sec), amplitude fluc, freq fluc
pub type HarmF = (f32,f32,Fluctuation,Fluctuation);
pub type Instrument = Vec<HarmF>;

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

pub fn learn_instrument(sample: &[i16], sr: usize, hz: f32) -> Instrument{
    let hs = 20;
    let hsf = hs as f32;
    let len = sample.len() as f32 / sr as f32;
    let mut instr = vec![(1. / hsf, len, (1., 0.), (1., 0.)); hs];
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
    pub fn mut_amp(harm: HarmF, delta: f32) -> HarmF{
        (harm.0 + delta, harm.1, harm.2, harm.3)
    }
    pub fn mut_len(harm: HarmF, delta: f32) -> HarmF{
        (harm.0, harm.1 + delta, harm.2, harm.3)
    }
    let mut error = std::usize::MAX;
    let mut rng = rand::thread_rng();
    let mut i = 0;
    loop{
        let old_error = error;
        for _ in 0..100{
            let h = choose(&mut rng, hs);
            let mut inst_clone = instr.clone();
            wchoose!(&mut rng, &[0.3, 0.3, 0.2, 0.2],
                {inst_clone[h] = mut_amp(inst_clone[h], 0.05)},
                {inst_clone[h] = mut_amp(inst_clone[h], -0.05)},
                {inst_clone[h] = mut_len(inst_clone[h], 0.05)},
                {inst_clone[h] = mut_len(inst_clone[h], -0.05)}
            );
            let tone = tone(hz, sr, len, inst_clone.clone());
            let err = cerror(&sample, &tone);
            if err < error{
                error = err;
                instr = inst_clone;
            }
            i += 1;
        }
        println!("---- {}, ", error);
        if old_error == error{
            break;
        }
    }
    println!("Did {} iterations!", i);
    instr
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
