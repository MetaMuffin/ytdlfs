
use std::ffi::OsStr;
use crate::indode_of_path;
use std::process::Command;
use fuse::ReplyDirectory;
use fuse::{FileType};


pub fn playlist_url(id: &String) -> String {
    return format!("https://www.youtube.com/playlist?list={0}",id);
}

pub fn playlist_dir_reply(mut reply: ReplyDirectory, offset: i64, pl_id: &String) {
    let mut entries = vec![
        (2, FileType::Directory, String::from(".")),
        (2, FileType::Directory, String::from("..")),
    ];
    let url = playlist_url(pl_id);
    let vids = get_playlist_elements(&url);

    for v in vids {
        let v_ent = (indode_of_path(OsStr::new(&v)),FileType::Symlink, v);
        entries.push(v_ent)
    }

    for (i, entry) in entries.into_iter().enumerate().skip(offset as usize) {
        reply.add(entry.0, (i + 1) as i64, entry.1, entry.2);
    }
    reply.ok();
}

pub fn get_playlist_elements(url: &String) -> Vec<String> {
    let mut comm = Command::new("/bin/youtube-dl");
    comm.arg("--flat-playlist").arg("--dump-json").arg(url);
    let out = comm.output().expect("Could not capture output of youtube_dl");
    let mut vids = Vec::new();

    for line in String::from_utf8(out.stdout).expect("Could not convert output to utf8 string").split("\n") {
        if line.len() < 1 {continue;}
        let j = json::parse(line).expect("Could not parse json output of youtube_dl"); 
        vids.push(j["url"].to_string())
    }

    return vids;
}