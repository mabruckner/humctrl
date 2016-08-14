use std::f32;
use std::mem;

pub fn bad_dft(signal: &Vec<(f32, f32)>) -> Vec<(f32, f32)>
{
    let mut output = Vec::new();
    for i in 0..signal.len() {
        output.push((0.0, 0.0));
    }
    sub_dft(signal, &mut output, 0, 1);
    output
}

fn sub_dft(signal: &Vec<(f32, f32)>, target: &mut Vec<(f32, f32)>, offset: usize, stride: usize) -> ()
{
    let frames = signal.len()/stride;
    let pi = f32::consts::PI;
    for i in 0..frames {
        let mut acc = (0.0, 0.0);
        for j in 0..frames {
            let index = j*stride + offset;
            let v = -2.0 * pi * (i * j) as f32 / frames as f32;
            let (a, b) = (v.cos(), v.sin());
            acc = (acc.0 + a*signal[index].0 - b*signal[index].1,
                   acc.1 + b*signal[index].0 + a*signal[index].1);
        }
        target[i*stride + offset] = acc;
    }
}

// offset and stride are for the output dft
fn collapse_composite(input: &Vec<(f32, f32)>, output: &mut Vec<(f32, f32)>, offset: usize, stride: usize) -> ()
{
    let frames = input.len()/stride;
    let pi = f32::consts::PI;
    for i in 0..(frames/2) {
        let v = -2.0 * pi * i as f32 / frames as f32;
        let (a, b) = (v.cos(), v.sin());
        let e = input[stride*i*2 + offset];
        let o = input[stride*i*2 + stride + offset];
        output[i*stride + offset] = (e.0 + o.0*a - o.0*b, e.1 + o.0*b + o.1*a);
        output[(i+frames/2)*stride + offset] = (e.0 - o.0*a + o.0*b, e.1 - o.0*b - o.1*a);
    }
}

pub fn better_dft(signal: &Vec<(f32, f32)>) -> Vec<(f32, f32)>
{
    let mut subdivision = 128;
    let mut output = Vec::new();
    let mut scratch = Vec::new();
    for i in 0..signal.len() {
        output.push((0.0, 0.0));
        scratch.push((0.0, 0.0));
    }
    for i in 0..subdivision {
        sub_dft(signal, &mut output, i, subdivision);
    }
    while subdivision > 1 {
        subdivision /= 2;
        for i in 0..subdivision {
            collapse_composite(&output, &mut scratch, i, subdivision);
        }
        mem::swap(&mut output, &mut scratch);
    }
    output
}
