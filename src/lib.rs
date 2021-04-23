use std::f32::consts::PI;
pub type Fluctuation = (f32,f32); // sin period(sec), sin amplitude
// property of harmonic: amplitude, duration to ampl = 0(sec), amplitude fluc, freq fluc
pub type Instrument = Vec<(f32,f32,Fluctuation,Fluctuation)>;

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
    ((t * 2f32 * PI / p).sin() * a)
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
