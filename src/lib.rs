use std::f32::consts::PI;

pub fn tone(hz: f32, sr: usize, len: f32) -> Vec<f32>{
    let mut samples = Vec::new();
    for s in 0..(len * sr as f32) as usize{
        let t = s as f32 / sr as f32;
        let mut s = 0.0;
        let mut amp = 0.0;
        for i in 1..20{
            let ohz = hz * i as f32;
            if ohz > 20000.0 { break; }
            let a = 1.0 / i as f32;
            s += sine_sample(t, ohz) * a;
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
