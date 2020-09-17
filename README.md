# ytdlfs

A Lnux FUSE Filesystem driver to mount youtube.

## Notice

This project is still WIP. The filesystem is *very* slow and has a lot of bugs.

## Usage

Mount filesystem with `cargo run [--release] <path>`

## Paths

Path | Description
---|---
`/v/<id>` | Video file by youtube video id
`/p/<id>` | Folder containing all videos of a youtube playlist by id
`/c/<id>` | TODO
`/search/<query>` | TODO
