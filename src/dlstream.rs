

use std::sync::Arc;
use std::process::ChildStdout;
use std::sync::Mutex;
use std::collections::HashMap;
use std::cmp::min;
use std::{io::{Read, BufReader}, process::{Stdio, Command}};
use fuse::ReplyData;

const MAX_CACHED: usize = 10;

lazy_static! {
    static ref STREAMS: Arc<Mutex<HashMap<String,DlStream>>> = {
        let m = HashMap::new();
        return Arc::new(Mutex::new(m));
    };
    static ref CACHED: Arc<Mutex<Vec<String>>> = {
        let m = Vec::new();
        return Arc::new(Mutex::new(m));
    };
}

pub struct DlStream {
    pub reader: BufReader<ChildStdout>,
    pub content: Vec<u8>
}

impl DlStream {
    pub fn new(url: &String) -> Result<DlStream, std::io::Error> {
        let mut comm = Command::new("/bin/youtube-dl");
        comm.arg("-f").arg("250").arg(url).arg("-o").arg("-").stdout(Stdio::piped());
        let proc = comm.spawn()?;
        let out = proc.stdout.unwrap();
        let reader = BufReader::new(out);
    
        return Ok(DlStream {
            reader,
            content: Vec::new()
        })
    }
    pub fn read_all(&mut self) {
        self.reader.read_to_end(&mut self.content).expect("Could not reader buffer");
    }
}


pub fn reply_read(reply: ReplyData, url: String, offset: i64, size: u32) {
    let mut streams_lock = STREAMS.lock().unwrap();
    let mut cached_lock = CACHED.lock().unwrap();
    if let None = streams_lock.get(&url) {
        let mut stream = DlStream::new(&url).expect("Could not create download stream");
        println!("Downloading...");
        stream.read_all();
        println!("Done");
        streams_lock.insert(url.clone(), stream);
        cached_lock.insert(0,url.clone());
        if cached_lock.len() > MAX_CACHED {
            let e = cached_lock.pop();
            if let Some(s) = e {
                streams_lock.remove(&s);
                println!("Cleaned up {0}",s);
            }
        } 
    }
    let stream = streams_lock.get(&url).unwrap();

    let start = offset as usize;
    let stop = min(start + (size as usize),stream.content.len());
    
    reply.data(&stream.content[start..stop]);
}

