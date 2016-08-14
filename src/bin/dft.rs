extern crate humctrl;

use humctrl::{bad_dft, better_dft};

fn main()
{
    let mut data = Vec::new();
    for i in 0..256 {
        if i % 2 == 0 {
            data.push((1.0,0.0));
        } else {
            data.push((0.0,0.0));
        }
    }
    let bad = bad_dft(&data);
    let better = better_dft(&data);

    for i in 0..data.len() {
        println!("{}\t{}", bad[i].0 - better[i].0, bad[i].1 - better[i].1);
    }
}
