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
