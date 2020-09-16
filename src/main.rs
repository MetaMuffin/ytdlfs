use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering::SeqCst;

use std::sync::Mutex;
use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::time::{Duration, UNIX_EPOCH};
use libc::{ENOENT,ENETUNREACH};
use fuse::{FileType, FileAttr, Filesystem, Request, ReplyData, ReplyEntry, ReplyAttr, ReplyDirectory};

mod dlstream;

#[macro_use]
extern crate lazy_static;

const TTL: Duration = Duration::from_secs(1);           // 1 second

const DIR_ATTR: FileAttr = FileAttr {
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

lazy_static! {
    static ref INODES: Mutex<HashMap<u64, String>> = {
        let m = HashMap::new();
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
                blocks: 2000,
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
            1 => reply.attr(&TTL, &DIR_ATTR),
            2 => reply.attr(&TTL, &HELLO_TXT_ATTR),
            _ => reply.error(ENOENT),
        }
    }

    fn read(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, size: u32, reply: ReplyData) {
        let map = INODES.lock().unwrap();
        let id = map.get(&ino);
        let url = format!("https://www.youtube.com/watch?v={0}", match id {
            Some(i) => {
                println!("Request for '{0}'",i);
                i
            },
            None => {
                reply.error(ENETUNREACH);
                return
            }
        });
        dlstream::reply_read(reply, url, offset, size);
       
    }

    fn readdir(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, mut reply: ReplyDirectory) {
        if ino != 1 {
            reply.error(ENOENT);
            return;
        }

        let entries = vec![
            (1, FileType::Directory, "."),
            (1, FileType::Directory, ".."),
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
    let options = ["-o", "ro", "-o", "fsname=ytdlfs"]
        .iter()
        .map(|o| o.as_ref())
        .collect::<Vec<&OsStr>>();
    fuse::mount(Ytdlfs, &mountpoint, &options).unwrap();
}
