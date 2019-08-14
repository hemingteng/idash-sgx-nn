use std::mem::swap;
use std::io::{BufReader, BufRead, Cursor, Read};
use std::fs::File;
use std::mem::size_of;
use std::net::TcpStream;
use byteorder::{ReadBytesExt,NativeEndian};


pub trait WeightsBuffer {
    fn getn(&mut self, n: usize) -> Vec<f32>;
}

pub struct TestWeightsBuffer {
    weights: Vec<f32>,
}

#[allow(dead_code)]
impl TestWeightsBuffer {
    pub fn new(weights: Vec<f32>) -> Self {
        Self { weights }
    }
}

impl WeightsBuffer for TestWeightsBuffer {
    fn getn(&mut self, n: usize) -> Vec<f32> {
        let mut out = self.weights.split_off(n);
        swap(&mut out, &mut self.weights);
        out
    }

}

#[allow(dead_code)]
pub struct RandomWeightsBuffer {
    index: usize,
}

impl RandomWeightsBuffer {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self { index: 1 }
    }
}

impl WeightsBuffer for RandomWeightsBuffer {
    fn getn(&mut self, n: usize) -> Vec<f32> {
        let i = self.index;
        self.index += n;
        (i..(i+n)).map(|x| (x*37%29/100) as f32).collect()
    }
}

#[allow(dead_code)]
pub struct FileWeightsBuffer {
    buffer: Box<dyn BufRead>,
}

impl FileWeightsBuffer {
    #[allow(dead_code)]
    pub fn new(filename: &str) -> Self {
        let f = File::open(filename).unwrap();
        Self { buffer: Box::new(BufReader::new(f)) as Box<dyn BufRead> }
    }

    #[allow(dead_code)]
    pub fn with_capacity(capacity: usize, filename: &str) -> Self {
        let f = File::open(filename).unwrap();
        Self { buffer: Box::new(BufReader::with_capacity(capacity, f)) 
            as Box<dyn BufRead> }
    }
}

impl WeightsBuffer for FileWeightsBuffer{
    fn getn(&mut self, n: usize) -> Vec<f32> {
        let mut out = Vec::<f32>::with_capacity(n);
        let mut buf = vec![0u8; size_of::<f32>()*n];
        self.buffer.read_exact(&mut buf).unwrap();
        let mut rdr = Cursor::new(buf);
        for _ in 0..n {
            out.push(rdr.read_f32::<NativeEndian>().unwrap());
        }
        out
    }
}

pub struct TcpWeightsBuffer {
    stream: TcpStream, 
}

#[allow(dead_code)]
impl TcpWeightsBuffer {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }
}

impl WeightsBuffer for TcpWeightsBuffer {
    fn getn(&mut self, n: usize) -> Vec<f32> {
        let mut out = Vec::<f32>::with_capacity(n);
        let mut buf = vec![0u8; size_of::<f32>()*n];
        self.stream.read_exact(&mut buf).unwrap();
        let mut rdr = Cursor::new(buf);
        for _ in 0..n {
            out.push(rdr.read_f32::<NativeEndian>().unwrap());
        }
        out
    }
}

pub struct MemTcpWeightsBuffer {
    weights: Vec<f32>,
    cursor: usize,
}

impl MemTcpWeightsBuffer {
    pub fn new(stream: TcpStream) -> Self {
        let mut stream = BufReader::new(stream);
        let mut weights = Vec::new();
        let mut next = stream.read_f32::<NativeEndian>();
        while next.is_ok() {
            weights.push(next.unwrap());
            next = stream.read_f32::<NativeEndian>();
        }
        Self { weights, cursor: 0 }
    }
}

impl WeightsBuffer for MemTcpWeightsBuffer {
    fn getn(&mut self, n: usize) -> Vec<f32> {
        let out = Vec::from(&self.weights[self.cursor..(self.cursor+n)]);
        self.cursor+=n;
        if self.cursor == self.weights.len() {
            self.cursor = 0;
        }
        out
    }
}