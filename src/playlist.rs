
use fuse::ReplyDirectory;
use fuse::{FileType};


pub fn playlist_url(id: &String) -> String {
    return format!("https://www.youtube.com/playlist?list={0}",id);
}

pub fn playlist_dir_reply(mut reply: ReplyDirectory, offset: i64, pl_id: &String) {
    let mut entries = vec![
        (2, FileType::Directory, "."),
        (2, FileType::Directory, ".."),
    ];
    let url = playlist_url(pl_id);
    
    

    for (i, entry) in entries.into_iter().enumerate().skip(offset as usize) {
        reply.add(entry.0, (i + 1) as i64, entry.1, entry.2);
    }
    reply.ok();
}