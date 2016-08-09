use std::f32;

pub fn bad_dft(signal: &Vec<(f32, f32)>) -> Vec<(f32, f32)>
{
    let mut output = Vec::new();
    let pi = f32::consts::PI;
    for i in 0..signal.len() {
        let mut acc = (0.0, 0.0);
        for j in 0..signal.len() {
            let v = -2.0 * pi * (i * j) as f32 / signal.len() as f32;
            let (a, b) = (v.sin(), v.cos());
            acc = (acc.0 + a*signal[j].0 - b*signal[j].1,
                   acc.1 + b*signal[j].0 + a*signal[j].1);
        }
        output.push(acc);
    }
    output
}


#[test]
fn it_works() {
}
