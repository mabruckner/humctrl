use portaudio as pa;
use std::cmp;
use std::ops::Deref;

struct Rotation<S: pa::Sample + 'static> {
    jump: usize,
    width: usize,
    vlist: Vec<Box<[S]>>,
    pos: usize
}

impl <S: pa::Sample + 'static> Rotation<S>
{
    fn append(&mut self, sample: S) {
        for i in 0..self.vlist.len() {
            if self.pos >= self.jump*i {
                self.vlist[i][self.pos-(self.jump*i)] = sample;
            } else {
                break
            }
        }
        self.pos += 1;
    }

    fn rotate(&mut self) {
        for i in 0..(self.vlist.len()-1) {
            self.vlist.swap(i,i+1);
        }
        self.pos -= self.jump;
    }

    fn init(&mut self, val: S)
    {
        for i in 0..(self.width/self.jump) {
            let mut v = Vec::with_capacity(self.width);
            for i in 0..self.width {
                v.push(val);
            }
            self.vlist.push(v.into_boxed_slice());
        }
    }
}

pub struct IStream<S: pa::Sample + 'static>
{
    rot: Rotation<S>,
    stream: pa::Stream<pa::Blocking<pa::stream::Buffer>,pa::Input<S>>
}

impl <S: pa::Sample + 'static> IStream<S> {
    pub fn new(stream: pa::Stream<pa::Blocking<pa::stream::Buffer>,pa::Input<S>>, width: usize, jump: usize) -> IStream<S>
    {
        IStream {
            rot: Rotation {
                jump: jump,
                width: width,
                vlist: Vec::new(),
                pos: 0
            },
            stream: stream
        }
    }

    /*fn append(&mut self, sample: S) {
        for i in 0..self.vlist.len() {
            if self.pos >= self.jump*i {
                self.vlist[i][self.pos-(self.jump*i)] = sample;
            } else {
                break
            }
        }
        self.pos += 1;
    }

    fn rotate(&mut self) {
        for i in 0..(self.vlist.len()+1) {
            self.vlist.swap(i,i+1);
        }
        self.pos -= self.jump;
    }

    fn init(vlist: &mut Vec<Box<[S]>>, width:usize, jump:usize, val: S) {
        for i in 0..(width/jump) {
            let mut v = Vec::with_capacity(width);
            for i in 0..width {
                v.push(val);
            }
            vlist.push(v.into_boxed_slice());
        }
    }*/

    pub fn get_next(&mut self) -> Option<&[S]>
    {
        if self.rot.vlist.len() == 0 {
            match self.stream.read(self.rot.width as u32) {
                Ok(samples) => {
                    println!("Forced read");
                    self.rot.init(samples[0]);
                    for i in 0..self.rot.width {
                        self.rot.append(samples[i]);
                    }
                    self.rot.rotate();
                    Some(self.rot.vlist.last().unwrap().deref())
                },
                Err(e) => None
            }
        } else {
            match self.stream.read_available() {
                Ok(pa::stream::Available::Frames(num)) if num > 0 => {
                    let to_read = cmp::min(num as usize, self.rot.width - self.rot.pos);
                    match self.stream.read(to_read as u32) {
                        Ok(samples) => {
                            println!("Free read {}", to_read);
                            for i in 0..to_read {
                                self.rot.append(samples[i]);
                            }
                            if self.rot.pos == self.rot.width {
                                self.rot.rotate();
                                Some(self.rot.vlist.last().unwrap().deref())
                            } else {
                                None
                            }
                        }
                        Err(e) => None
                    }
                },
                _ => None
            }
        }
    }
    pub fn into_stream(self) -> pa::Stream<pa::Blocking<pa::stream::Buffer>,pa::Input<S>>
    {
        self.stream
    }
}
