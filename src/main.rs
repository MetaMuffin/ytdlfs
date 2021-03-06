

use libc::ENETUNREACH;
use std::sync::Mutex;
use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::{time::{Duration, UNIX_EPOCH}};
use libc::{ENOENT};
use fuse::{FileType, FileAttr, Filesystem, Request, ReplyData, ReplyEntry, ReplyAttr, ReplyDirectory};

mod video;
mod channel;
mod playlist;
mod helper;
use helper::indode_of_path;

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



struct Ytdlfs;

pub enum FsContent {
    Video(String,bool), // id, video
    ChannelDir(String),
    PlaylistDir(String,Option<String>, bool),
}

lazy_static! {
    static ref INODES: Mutex<HashMap<u64, FsContent>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };
}


impl Filesystem for Ytdlfs {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        println!("Request for inode of {:?} from {}",name,parent);
        let mut map = INODES.lock().unwrap();
        if parent == 1 {
            let ino = match name.to_str().expect("Could not convert OsStr to str") {
                "a" => 2,
                "v" => 3,
                "pa" => 4,
                "pv" => 5,
                _ => {
                    return reply.error(ENOENT);
                }
            };
            let attr = FileAttr {
                ino: ino, size: 4096, blocks: 1,
                atime: UNIX_EPOCH, mtime: UNIX_EPOCH, ctime: UNIX_EPOCH, crtime: UNIX_EPOCH,
                kind: FileType::Directory, perm: 0o644, nlink: 1, uid: 501, gid: 20, rdev: 0, flags: 0,
            };
            reply.entry(&TTL, &attr, 0);
        } else if parent == 2 || parent == 3{
            let video = parent == 3;
            let ino = indode_of_path(name) + (if video { 0 } else { 1 }); // Change the hash, so that it doesnt collide with the audio version
            map.insert(ino, FsContent::Video(String::from(name.to_str().unwrap()),video));
            
            let attr = FileAttr {
                ino: ino, size: 1 << 31, blocks: 1,
                atime: UNIX_EPOCH, mtime: UNIX_EPOCH, ctime: UNIX_EPOCH, crtime: UNIX_EPOCH,
                kind: FileType::RegularFile, perm: 0o644, nlink: 1, uid: 501, gid: 20, rdev: 0, flags: 0,
            };
            reply.entry(&TTL, &attr, 0);
        } else if parent == 4 || parent == 5 { // A playlist folder
            let video = parent == 5;
            let ino = indode_of_path(name) + (if video { 0 } else { 1 });
            
            let id_and_ext = String::from(name.to_str().unwrap());
            let (id,file_ext) = match id_and_ext.find("#") {
                Some(mid) => {
                    let (a,b_with_delimeter) = id_and_ext.split_at(mid);
                    let b = b_with_delimeter.trim_start_matches(".");
                    (String::from(a),Some(String::from(b)))
                },
                None => (id_and_ext, None),
            };
            map.insert(ino, FsContent::PlaylistDir(id,file_ext,video));
            
            let attr = FileAttr {
                ino: ino, size: 4096, blocks: 1,
                atime: UNIX_EPOCH, mtime: UNIX_EPOCH, ctime: UNIX_EPOCH, crtime: UNIX_EPOCH,
                kind: FileType::Directory, perm: 0o644, nlink: 1, uid: 501, gid: 20, rdev: 0, flags: 0,
            };
            reply.entry(&TTL, &attr, 0);
        } else {
            let mut content_o = None;
            {
                content_o = map.get(&parent);
            }
            if let Some(content) = content_o {
                if let FsContent::PlaylistDir(pl_id, file_ext, video) = content {
                    
                    let ino = indode_of_path(name) + (if *video { 0 } else { 1 });
                    let vid = video.clone();
                    map.insert(ino, FsContent::Video(String::from(name.to_str().unwrap()),vid));
                    
                    let attr = FileAttr {
                        ino: ino, size: 1000000, blocks: 1,
                        atime: UNIX_EPOCH, mtime: UNIX_EPOCH, ctime: UNIX_EPOCH, crtime: UNIX_EPOCH,
                        kind: FileType::RegularFile, perm: 0o644, nlink: 1, uid: 501, gid: 20, rdev: 0, flags: 0,
                    };
                    reply.entry(&TTL, &attr, 0);
                } else {
                    reply.error(ENOENT)
                }
            } else {
                reply.error(ENOENT)
            }
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        match ino {
            1 => reply.attr(&TTL, &DIR_ATTR),
            _ => reply.error(ENOENT),
        }
    }

    fn read(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, size: u32, reply: ReplyData) {
        println!("Read of inode {0}",ino);
        let map = INODES.lock().unwrap();
        let id = map.get(&ino);
        match id {
            Some(c) => {
                match c {
                    FsContent::Video(i,video) => {
                        if i.len() != 11 { return reply.error(ENETUNREACH) };
                        println!("\tthats {0} (video: {1})",i,video);
                        video::video_reply(reply, i, offset, size,*video)
                    },
                    _ => return reply.error(ENETUNREACH)
                }
            },
            None => {
                reply.error(ENOENT);
                return
            }
        }
       
    }

    fn readdir(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, mut reply: ReplyDirectory) {
        if ino != 1 {
            let map = INODES.lock().unwrap();
            if let Some(content) = map.get(&ino) {
                if let FsContent::PlaylistDir(ch_id, file_ext,video) = content {
                    if ch_id.len() != 34 { return reply.error(ENOENT) };
                    playlist::playlist_dir_reply(reply, offset, ch_id, file_ext, video);
                } else { reply.error(ENOENT) }
            } else { reply.error(ENOENT) }
        } else {
            let entries = vec![
                (1, FileType::Directory, "."),
                (1, FileType::Directory, ".."),
            ];
    
            for (i, entry) in entries.into_iter().enumerate().skip(offset as usize) {
                reply.add(entry.0, (i + 1) as i64, entry.1, entry.2);
            }
            reply.ok();
        }

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
