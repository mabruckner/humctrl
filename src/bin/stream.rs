extern crate portaudio;
extern crate humctrl;
extern crate graph;
extern crate clap;

use graph::GridPrint;

use portaudio as pa;
use clap::{App, Arg};

use humctrl::stream::IStream;

fn main() {
    let matches = App::new("stream")
                        .about("Analyzes real time audio data")
                        .arg(Arg::with_name("height")
                             .short("h")
                             .long("height")
                             .value_name("HEIGHT")
                             .help("The height of the display in characters."))
                        .arg(Arg::with_name("width")
                             .short("w")
                             .long("width")
                             .value_name("WIDTH")
                             .help("The height of the display in characters."))
                        .arg(Arg::with_name("harmonics")
                             .short("n")
                             .long("harmonics")
                             .value_name("HARMONICS")
                             .help("expected number of hamonics (including fundamental) for use in the harmonic product spectrum. 3 is good for voice, 1 is good for a tuning fork."))
                        .get_matches();

    let (width, height, harmonics) = (128, 50, 1);
    let harmonics = match matches.value_of("harmonics") {
        Some(n) => n.parse::<usize>().unwrap_or(harmonics),
        None => harmonics
    };
    let width = match matches.value_of("width") {
        Some(w) => w.parse::<usize>().unwrap_or(width),
        None => width
    };
    let height = match matches.value_of("height") {
        Some(h) => h.parse::<usize>().unwrap_or(height),
        None => height
    };

    let mut g = graph::Graph::hist(width*2, height, Box::new(|&(x,y): &(f32,f32)| {
        (x*x + y*y).sqrt()/100.0
    }));
    let t_res = 50;
    let mut t = graph::Graph::scatter(t_res, height, Box::new(move |&(i, y)| {
        i as f32 / t_res as f32
    }), Box::new(|&(i, y)| {
        y as f32 / 500.0 - 0.1
    }));
    let context = pa::PortAudio::new().unwrap();

    let sample_rate = 48000.0;

    let mut settings = context.default_input_stream_settings::<f32>(1, sample_rate, 2048).unwrap();
    settings.flags = pa::stream_flags::CLIP_OFF;
    print!("settings: {:?}\n", settings);

    let mut stream = context.open_blocking_stream(settings).unwrap();

    println!("Starting");
    stream.start().unwrap();
    println!("Started");

    let width = 1024;

    let mut input = IStream::new(stream, width, 512);
    let mut vec = Vec::new();
    for i in 0..width {
        vec.push((0.0,0.0));
    }
    let mut points = Vec::new();
    loop {
        let mut i = 0;
        loop {
            i+= 1;
            if i% 10000 == 0 {
                println!("!");
            }
            if let Some(thing) = input.get_next() {
                if i == 1 {
                    println!("SKIP");
                    continue
                }
                println!("Found! {}",i);
                for i in 0..width {
                    vec[i] = (thing[i], 0.0);
                }
                let sig = humctrl::pad_zero(&vec, width*3);
                let frames = sig.len();
                let dft = humctrl::better_dft(&sig);

                let mut v = Vec::new();
                let product = humctrl::harmonic_product(&humctrl::first_half(&dft), harmonics);
                let (start, end) = (2, 256);
                let (maxval, _) = product.iter().take(end).enumerate().skip(start).fold((0, 10.0), |(ai, am), (bi, &(bx, by))| {
                    let bm = (bx*bx+by*by).sqrt();
                    if bm > am {
                        (bi, bm)
                    } else {
                        (ai, am)
                    }
                });
                for i in 0..256 {
                    v.push(dft[i]);
                }
                points.push(match maxval {
                    0 => None,
                    x => Some(x as f32 * sample_rate as f32 / frames as f32 )
                });
                if points.len() > t_res {
                    points.remove(0);
                }
                g.set_data(v);
                t.set_data(points.iter().enumerate().filter_map(|(i, v)| {
                    match v {
                        &Some(val) => Some((i,val)),
                        &None => None
                    }
                }).collect());
                println!("\x1b[0;0f");
                g.print();
                println!("\x1b[K{:?} Hz", maxval as f32 * sample_rate as f32 / frames as f32 );
                t.print();
                print!("\x1b[0;0f");
                
                //println!("{:?}", thing);
                break
            }
        }
    }

    let mut stream = input.into_stream();

    println!("Stopping");
    stream.stop().unwrap();
    println!("Closing");
    stream.close().unwrap();
    println!("Done");
    context.terminate();

}
