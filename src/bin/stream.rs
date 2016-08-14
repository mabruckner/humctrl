extern crate portaudio;
extern crate humctrl;
extern crate graph;

use graph::GridPrint;

use portaudio as pa;

use humctrl::stream::IStream;

fn main() {
    let mut g = graph::Graph::hist(256,60, Box::new(|&(x,y): &(f32,f32)| {
        (x*x + y*y).sqrt()/100.0
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
                let dft = humctrl::better_dft(&sig);

                let mut v = Vec::new();
                let product = humctrl::harmonic_product(&humctrl::first_half(&dft), 1);
                for i in 0..256 {
                    v.push(product[i]);
                }
                g.set_data(v);
                println!("\x1b[0;0f");
                g.print();
                println!("\x1b[0;0f");
                
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
