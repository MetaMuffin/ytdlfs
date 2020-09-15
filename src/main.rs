use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::atomic::AtomicI64;
use std::sync::Mutex;
use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::time::{Duration, UNIX_EPOCH};
use libc::{ENOENT,ENETUNREACH};
use fuse::{FileType, FileAttr, Filesystem, Request, ReplyData, ReplyEntry, ReplyAttr, ReplyDirectory};
use std::process::Command;
use std::cmp::min;

#[macro_use]
extern crate lazy_static;

const TTL: Duration = Duration::from_secs(1);           // 1 second

const HELLO_DIR_ATTR: FileAttr = FileAttr {
    ino: 1,
    size: 0,
    blocks: 0,
    atime: UNIX_EPOCH,                                  // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::Directory,
    perm: 0o755,
    nlink: 2,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
};


const HELLO_TXT_ATTR: FileAttr = FileAttr {
    ino: 2,
    size: 1000000,
    blocks: 22,
    atime: UNIX_EPOCH,                                  // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::RegularFile,
    perm: 0o644,
    nlink: 1,
    uid: 501,
    gid: 20,
    rdev: 0,
    flags: 0,
};

struct Ytdlfs;
/*
fn b64d(s:&str) -> Option<u64> {
    let mut i: u64 = 0;
    let mut count = 0;
    let alpha = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-_";
    for c in s.chars() {
        let v = match alpha.find(c) {
            Some(v) => v,
            None => { return None }
        } as u32;
        i += (v * u32::pow(alpha.len() as u32,v)) as u64;
        count += 1;
    }
    return Some(i);
}
fn b64e(i:i64) -> String {
    
}*/

lazy_static! {
    static ref INODES: Mutex<HashMap<u64, String>> = {
        let mut m = HashMap::new();
        
        Mutex::new(m)
    };
}

static COUNTER: AtomicU64 = AtomicU64::new(5);

impl Filesystem for Ytdlfs {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        if parent == 1 {
            COUNTER.fetch_add(1, SeqCst);
            let ino = COUNTER.load(SeqCst);
            let mut map = INODES.lock().unwrap();
            map.insert(ino, String::from(name.to_str().unwrap()));

            
            let attr = FileAttr {
                ino: ino,
                size: 1000000,
                blocks: 22,
                atime: UNIX_EPOCH,
                mtime: UNIX_EPOCH,
                ctime: UNIX_EPOCH,
                crtime: UNIX_EPOCH,
                kind: FileType::RegularFile,
                perm: 0o644,
                nlink: 1,
                uid: 501,
                gid: 20,
                rdev: 0,
                flags: 0,
            };
            reply.entry(&TTL, &attr, 0);
        } else {
            reply.error(ENOENT);
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        match ino {
            1 => reply.attr(&TTL, &HELLO_DIR_ATTR),
            2 => reply.attr(&TTL, &HELLO_TXT_ATTR),
            _ => reply.error(ENOENT),
        }
    }

    fn read(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, size: u32, reply: ReplyData) {
        
        let mut pro = Command::new("/bin/youtube-dl");
        let map = INODES.lock().unwrap();
        let id = map.get(&ino);
        pro.arg("-f").arg("250").arg(format!("https://www.youtube.com/watch?v={0}",id)).arg("-o").arg("-");
        let data = match pro.output() {
            Ok(d) => d.stdout,
            Err(_e) => return reply.error(ENETUNREACH),
        };
        if ino == 2 {
            let start = offset as usize;
            let stop = min(start + (size as usize), data.len());
            
            reply.data(&data[start..stop]);
        } else {
            reply.error(ENOENT);
        }
    }

    fn readdir(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, mut reply: ReplyDirectory) {
        if ino != 1 {
            reply.error(ENOENT);
            return;
        }

        let entries = vec![
            (1, FileType::Directory, "."),
            (1, FileType::Directory, ".."),
            (2, FileType::RegularFile, "test.webm"),
        ];

        for (i, entry) in entries.into_iter().enumerate().skip(offset as usize) {
            // i + 1 means the index of the next entry
            reply.add(entry.0, (i + 1) as i64, entry.1, entry.2);
        }
        reply.ok();
    }
}

fn main() {
    let mountpoint = env::args_os().nth(1).unwrap();
    let options = ["-o", "ro", "-o", "fsname=hello"]
        .iter()
        .map(|o| o.as_ref())
        .collect::<Vec<&OsStr>>();
    fuse::mount(Ytdlfs, &mountpoint, &options).unwrap();
}
