extern crate humctrl;
extern crate graph;

use graph::GridPrint;
use std::f32;

use std::thread::sleep;
use std::time::Duration;

fn main() {
    let mut g = graph::Graph::hist(256,40, Box::new(|&(x,y): &(f32,f32)| {
        (x*x + y*y).sqrt()/500.0
    }));
    let mut s = graph::Graph::hist(256,40, Box::new(|&(x,y): &(f32,f32)| {
        (x / 3.0) + 0.5
    }));
    let mut thing = 0.0;
    let width = 512;
    let mut vec = Vec::new();
    for i in 0..width {
        vec.push((0.0,0.0));
    }
    let sleepdur = Duration::from_millis(100);
    loop {
        thing += 1.0;
        for i in 0..width {
            vec[i] = ((2.0*f32::consts::PI*(i as f32)*thing/(width as f32)).cos(), 0.0);
        }
        let dft = humctrl::better_dft(&vec);
        g.set_data(dft);
        s.set_data(vec.clone());
        println!("\x1b[0;0f");
        g.print();
        s.print();
        println!("\x1b[0;0f");
        sleep(sleepdur);
    }
}
