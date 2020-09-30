

use regex::Regex;
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
    pub fn new(url: &String, video: bool) -> Result<DlStream, std::io::Error> {
        println!("Downloading from url {0}",url);
        let out = ytdl_stdout(url,video)?;
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

pub fn ytdl_stdout(id: &String, video: bool) -> Result<ChildStdout, std::io::Error> {
    let mut comm = Command::new("/bin/youtube-dl");
    if video {
        //comm.arg("-f").arg(format!("{0}",f)).arg(url).arg("-o").arg("-").stdout(Stdio::piped());
        comm.arg(id).arg("-o").arg("-").stdout(Stdio::piped());
    } else {
        let mut comm_ytdl = Command::new("/bin/youtube-dl");
        comm_ytdl.arg(id).arg("-o").arg("-").stdout(Stdio::piped());
        let ytdl_out = comm_ytdl.spawn().unwrap().stdout.unwrap();
        comm = Command::new("/bin/ffmpeg");
        comm.arg("-i").arg("-").arg("-vn").arg("-f").arg("wav").arg("-acodec").arg("copy").arg("-");
        comm.stdin(ytdl_out);
        comm.stdout(Stdio::piped());
    }
    let proc = comm.spawn()?;
    return Ok(proc.stdout.unwrap());
}

pub fn video_url(id: &String) -> String {
    return format!("https://www.youtube.com/watch?v={0}",id);
}

pub fn id_from_url(url: String) -> Option<String> {
    println!("{:?}",url);
    let re = Regex::new(r"https?://www\.youtube\.com/watch\?v=(.{12})").unwrap();
    if let Some(caps) = re.captures(&url) {
        let ret = caps.get(1).map_or(None, |m| Some(String::from(m.as_str())));
        println!("{:?}",ret);
        return ret;
    } else { None }
}


pub fn video_reply(reply: ReplyData, id: &String, offset: i64, size: u32, video: bool) {
    let url = video_url(&id);
    let mut streams_lock = STREAMS.lock().unwrap();
    let mut cached_lock = CACHED.lock().unwrap();
    if let None = streams_lock.get(&url) {
        let mut stream = DlStream::new(&url,video).expect("Could not create download stream");
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

    println!("Offset: {0}, size: {1}",offset,size);
    let start = offset as usize;
    let stop = min(start + (size as usize),stream.content.len());
    
    reply.data(&stream.content[start..stop]);
}

