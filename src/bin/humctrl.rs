extern crate portaudio;
extern crate humctrl;
extern crate hyper;
extern crate time;
use portaudio as pa;

use humctrl::{bad_dft, better_dft};
use std::sync::mpsc;
use std::env;
use hyper::Client;
use time::{Duration, PreciseTime};

#[derive(Debug, PartialEq, Eq)]
struct Interval(usize, usize);

impl Interval
{
    fn new(ratio: f32, error: f32) -> Interval
    {
        for d in 1..100 {
            let nf = d as f32 * ratio;
            if ((nf.round()/d as f32) - ratio).abs() <= error {
                return Interval(nf.round() as usize, d)
            }
        }
        return Interval(0, 0)
    }
}

fn recognize(unfiltered: &Vec<(f32, f32)>, error: f32, thresh: f32, pattern: &Vec<f32>) -> Option<Vec<usize>>
{
    let sounds = unfiltered.iter().filter(|x| { x.1 >= thresh }).collect::<Vec<&(f32,f32)>>();
    for i in 0..sounds.len() {
        let f = sounds[i].0;
        let mut loc = 0;
        let mut finds = vec![i];
        for j in (i+1)..sounds.len() {
            let diff = sounds[j].0 / f;
            if ((diff / pattern[loc]) - 1.0).abs() < error {
                loc += 1;
                finds.push(j);
            }
            if loc >= pattern.len() {
                return Some(finds);
            }
        }
    }
    None
}

#[test]
fn inttest() {
    assert!(Interval(4,5) == Interval::new(0.8,0.001));
    let x = Interval::new(128.9 / 140.6, 0.01);
    assert!(Interval(1,1) != x);
    assert!(Interval(0,0) != x);
}

fn intervals(sounds: &Vec<(f32, f32)>, error: f32, thresh: f32) -> Vec<Interval>
{
    let filtered = sounds.iter().filter(|x| { x.1 >= thresh }).collect::<Vec<&(f32,f32)>>();
    println!("{}, {:?}\n",filtered.len(), filtered);
    if filtered.len() == 0 {
        return Vec::new();
    }
    let mut prev = filtered[0].0;
    let mut out = Vec::new();
    for i in 1..filtered.len() {
        let int = Interval::new(filtered[i].0/prev, error);
        if int == Interval(0,0) || int == Interval(1,1) {
            continue;
        }
        out.push(int);
        prev = filtered[i].0;
    }
    out
}

fn plot_bar<I>(vals: I) where I: Iterator<Item=usize>
{
    for v in vals {
        for i in 0..v {
            print!("-");
        }
        println!("");
    }
}

fn to_decibel(pair: (f32, f32)) -> f32
{
    (pair.0*pair.0 + pair.1*pair.1).sqrt().log(10.0)*10.0
}

fn to_freq(dft: &Vec<(f32, f32)>, rate: f32) -> Vec<(f32, f32)>
{
    let base = rate / dft.len() as f32;
    let mut out = Vec::new();
    for i in 0..dft.len()/2 {
        let v = to_decibel(dft[i]);
        let f = base * i as f32;
        out.push((f,v));
    }
    out
}

fn peaks(components: &Vec<(f32, f32)>) -> Vec<(f32, f32)>
{
    let mut prev = components[0];
    let mut uphill = true;
    let mut out = Vec::new();
    for &(f, v) in components
    {
        if uphill && prev.1 > v {
            out.push(prev);
            uphill = false;
        } else if v > prev.1 {
            uphill = true;
        }
        prev = (f, v);
    }
    out
}

fn fundamental(peaks: &Vec<(f32, f32)>, vthresh: f32, fthresh: f32) -> Option<(f32, f32)>
{
    if peaks.len() == 0 {
        return None
    }
    let mut m = peaks[0];
    for &p in peaks {
        if p.1 > m.1 {
            m = p;
        }
    }
    let filtered = peaks.clone().into_iter().filter(|x| m.1 - x.1 <= vthresh);
    for (f, v) in filtered {
        if f == 0.0 {
            continue;
        }
        let n = m.0 / f;
        if (n.round() - n).abs() < fthresh {
            return Some((f,v))
        }
    }
    None
}

fn get_freq(dft: &Vec<(f32, f32)>, rate: f32) -> (f32, f32)
{
    let base = rate / dft.len() as f32;
    let mut out = None;
    for i in 0..dft.len()/2 {
        let v = to_decibel(dft[i]);
        let f = base * i as f32;
        if let Some((of, ov)) = out {
            if ov < v {
                out = Some((f,v));
            }
        } else {
            out = Some((f,v));
        }
    }
    out.unwrap()
}

fn trigger(body: &str, addr: &str, user: &str) -> ()
{
    let client = Client::new();
    println!("{:?}", client.put(&format!("http://{}/api/{}/groups/1/action", addr, user))
        .body(body).send().unwrap());
}

fn main(){
    let mut args = env::args();
    args.next();
    let addr = args.next().expect("need a hue bridge address");
    let user = args.next().expect("need a hue username");
    let context = pa::PortAudio::new().unwrap();
    let sample_rate = 48000.0 / 4.0;
    let target_off = vec![1.5, 1.2];
    let target_on = vec![1.5, 1.8];
    print!("NUM: {}\n", context.device_count().unwrap());
    for device in context.devices().unwrap() {
        print!("\t{:?}\n", device);
    }
    let mut settings = context.default_input_stream_settings::<f32>(1, sample_rate, 4096).unwrap();
        settings.flags = pa::stream_flags::CLIP_OFF;
    print!("settings: {:?}\n", settings);

    let mut stream = context.open_blocking_stream(settings).unwrap();

    println!("Starting");
    stream.start().unwrap();
    println!("Started");
    let size = 2048;
    let rate = 1;
    let samples = 10;
    let mut dat = Vec::new();
    for i in 0..size {
        dat.push((0.0, 0.0));
    }
    let mut big = (0.0,0.0);
    let mut list = Vec::new();
    for iteration in 0..79 {
        println!("{} {}", iteration, list.len());
        let t_0 = PreciseTime::now();
//        let vals = stream.read((size * rate) as u32).unwrap();
        let t_1 = PreciseTime::now();
        let vals = match stream.read_available().unwrap() {
            pa::stream::Available::Frames(num) => {
                println!("read {}", num);
                if (num as usize) < size {
                    stream.read(size as u32).unwrap()
                } else {
                    stream.read(size as u32).unwrap()
                }
            },
            _ => {
                println!("THERE WAS AN ERROR!");
                panic!("ERROR");
            }
        };
        let t_1_5 = PreciseTime::now();
        for i in 0..size {
            dat[i] = (vals[i*rate], 0.0);
        }
        let t_2 = PreciseTime::now();
        let dft = better_dft(&dat);
        let t_3 = PreciseTime::now();
        let (freq, vol) = get_freq(&dft, sample_rate as f32 / rate as f32);
//        plot_bar(dft.iter().map(|v|{ (10.0*(v.0*v.0 + v.1*v.1)).sqrt() as usize}));
        println!("{}Hz\t@\t{}dB",freq, vol);
        if vol > 1.0 {
            big = (freq, vol);
        }
        let freqs = to_freq(&dft, sample_rate as f32 / rate as f32);
        let spikes = peaks(&freqs).into_iter().filter(|x|{x.0 > 40.0}).collect();
        let fundamental = fundamental(&spikes, 8.0, 0.1);
        let t_4 = PreciseTime::now();
        println!("{}", list.len());
        if let Some(val) = fundamental {
            list.push(val.clone());
        }
        if list.len() > samples {
            list.remove(0);
        }
        if let Some(seq) = recognize(&list, 0.13, 9.0, &target_off) {
            println!("FOUND OFF");
            trigger("{\"on\": false}", &addr, &user);
            list = Vec::new();
        }
        if let Some(seq) = recognize(&list, 0.13, 9.0, &target_on) {
            println!("FOUND ON");
            trigger("{\"on\": true}", &addr, &user);
            list = Vec::new();
        }
        let t_5 = PreciseTime::now();
        println!("{}", list.len());
        println!("{}Hz\t@\t{}dB", big.0, big.1);
//        println!("{:?}",spikes);
        println!("{:?}", fundamental);
        println!("{:?}", intervals(&list, 0.01, 10.0));
        println!("{} {} {} {} {}", t_0.to(t_1), t_1.to(t_2), t_2.to(t_3), t_3.to(t_4), t_4.to(t_5));
        println!("{}", t_1.to(t_1_5));
    }
    stream.stop().unwrap();
    println!("Closing");
    stream.close().unwrap();
    println!("Done");
    context.terminate();

}
