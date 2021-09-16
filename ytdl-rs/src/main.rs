use anyhow::*;
use serde_json::{Value};
use youtube_dl_pyo3::YoutubeDl;
use std::fs;
use pyo3::{prelude::*, types::PyDict};

use ytdl_rs::YoutubeDl as YtDl;

fn main() -> Result<()> {
    let ytdl = YoutubeDl::options()
        .set_quiet(true)
        .set_extract_flat(true)
        .build();

    //let r = ytdl.extract_info("https://www.youtube.com/playlist?list=PLaN19gIKi5ZrnOIl7YrLgoDogYCaiy903")?;
    //let r = ytdl.extract_info("https://www.youtube.com/watch?v=fhCmHwNanZI")?;
    //let v: Value = serde_json::from_str(&r)?;

    let output = YtDl::new("https://www.youtube.com/playlist?list=PLaN19gIKi5ZrnOIl7YrLgoDogYCaiy903").flat_playlist(true).socket_timeout(10).run()?;
    //let output = YtDl::new("https://www.youtube.com/watch?v=fhCmHwNanZI").flat_playlist(true).socket_timeout(10).run()?;

    Ok(())
}
