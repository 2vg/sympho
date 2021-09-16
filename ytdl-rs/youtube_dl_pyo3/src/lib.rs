use pyo3::prelude::Python;
use pyo3::PyResult;
use pyo3::types::{PyString, PyDict, PyList};
use serde_json;

use serde_derive::*;
use from_py_dict_derive::*;
use from_py_dict::FromPyDict;

///
/// Represents a youtube-dl extractor
///
#[derive(Serialize, Deserialize, FromPyDict)]
pub struct Extractor {
    pub name: String,
    pub working: bool
}

///
/// Represents an item in a playlist
///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlaylistItem {
    pub _type: String,
    pub url: String,
    pub ie_key: String,
    pub id: String,
    pub title: Option<String>
}

///
/// Represents a playlist
///
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Playlist {
    pub _type: String,
    pub entries: Vec<PlaylistItem>,
    pub id: String,
    pub webpage_url: String,
    pub webpage_url_basename: String,
    pub extractor_key: String
}

///
/// Represents an audio / video format
///
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Format {
    pub acodec: Option<String>,
    pub ext: Option<String>,
    pub format_id: String,
    pub fps: Option<f64>,
    pub manifest_url: Option<String>,
    pub preference: Option<i32>,
    pub protocol: String,
    pub tbr: Option<f64>,
    pub url: String,
    pub vcodec: Option<String>
}

///
/// Represents a subtitle source
///
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Subtitle {
    pub ext: String,
    pub url: String
}

///
/// List of available subtitle sources
///
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Subtitles {
    pub rechat: Option<Vec<Subtitle>>
}

///
/// Represents a thumbnail source
///
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Thumbnail {
    pub id: String,
    pub url: String
}

///
/// Represents a video
///
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Video {
    pub acodec: Option<String>,
    pub description: Option<String>,
    pub display_id: Option<String>,
    pub duration: Option<f32>, // streams
    pub ext: Option<String>,
    pub extractor: String,
    pub extractor_key: String,
    pub format: Option<String>,
    pub format_id: String,
    pub formats: Option<Vec<Format>>,
    pub fps: Option<f64>,
    pub height: Option<i32>,
    pub id: String,
    pub is_live: Option<bool>,
    pub manifest_url: Option<String>,
    pub playlist: Option<String>,
    pub playlist_index: Option<i32>,
    pub preference: Option<i32>,
    pub protocol: Option<String>,
    pub requested_subtitles: Option<bool>,
    pub subtitles: Option<Subtitles>,
    pub tbr: Option<f64>,
    pub thumbnail: Option<String>, // Twitch
    pub thumbnails: Option<Vec<Thumbnail>>,
    pub timestamp: Option<i64>,
    pub title: String,
    pub upload_date: Option<String>,
    pub uploader: Option<String>,
    pub uploader_id: Option<String>,
    pub url: Option<String>,
    pub vcodec: Option<String>,
    pub view_count: Option<i32>,
    pub webpage_url: Option<String>,
    pub webpage_url_basename: Option<String>,
    pub width: Option<i32>
}

///
/// Options for youtube-dl.
/// The full list of options and their meaning is available in the [youtube-dl documentation](https://github.com/ytdl-org/youtube-dl/blob/master/youtube_dl/YoutubeDL.py#L141)
///
/// This struct should not be used directly in most cases.
/// You can use
/// ```
/// use youtube_dl::YoutubeDl;
///
/// YoutubeDl::options().set_quiet(true).build();
/// ```
/// instead, to create a YoutubeDl object with the wanted options.
///
#[derive(Clone, Debug, Default)]
pub struct YoutubeDlOptions {
    pub username: Option<String>,
    pub password: Option<String>,
    pub videopassword: Option<String>,
    pub ap_mso: Option<String>,
    pub ap_username: Option<String>,
    pub ap_password: Option<String>,
    pub usenetrc: Option<String>,
    pub verbose: Option<bool>,
    pub quiet: Option<bool>,
    pub no_warnings: Option<bool>,
    pub forceurl: Option<bool>,
    pub forcetitle: Option<bool>,
    pub forceid: Option<bool>,
    pub forcethumbnail: Option<bool>,
    pub forcedescription: Option<bool>,
    pub forcefilename: Option<bool>,
    pub forceduration: Option<bool>,
    pub forcejson: Option<bool>,
    pub dump_single_json: Option<bool>,
    pub simulate: Option<bool>,
    pub format: Option<String>,
    pub outtmpl: Option<String>,
    pub restrictfilenames: Option<bool>,
    pub ignoreerrors: Option<bool>,
    pub force_generic_extractor: Option<bool>,
    pub nooverwites: Option<bool>,
    pub playliststart: Option<i32>,
    pub playlistend: Option<i32>,
    pub playlist_items: Option<Vec<i32>>,
    pub playlistreverse: Option<bool>,
    pub playlistrandom: Option<bool>,
    pub matchtitle: Option<bool>,
    pub rejecttitle: Option<bool>,
    // pub logger
    pub logtostderr: Option<bool>,
    pub writedescription: Option<bool>,
    pub writeinfojson: Option<bool>,
    pub writeannotations: Option<bool>,
    pub writethumbnail: Option<bool>,
    pub write_all_thumbnail: Option<bool>,
    pub writesubtitles: Option<bool>,
    pub allsubtitles: Option<bool>,
    pub listsubtitles: Option<bool>,
    pub subtitlesformat: Option<String>,
    pub subtitleslangs: Option<Vec<String>>,
    pub keepvideo: Option<bool>,
    // pub daterange
    pub skip_download: Option<bool>,
    pub cachedir: Option<String>,
    pub noplaylist: Option<bool>,
    pub age_limit: Option<i32>,
    pub min_views: Option<i32>,
    pub max_view: Option<i32>,
    pub download_archive: Option<String>,
    pub cookiefile: Option<String>,
    pub nocheckcertificate: Option<bool>,
    pub prefer_insecure: Option<bool>,
    pub proxy: Option<String>,
    pub geo_verification_proxy: Option<String>,
    pub socket_timeout: Option<i32>,
    pub bidi_workaround: Option<bool>,
    pub debug_printtraffic: Option<bool>,
    pub include_ads: Option<bool>,
    pub default_search: Option<String>,
    pub encoding: Option<String>,
    pub extract_flat: Option<bool>,

    // pub postprocessors
    // pub progress_hooks
    pub merge_output_format: Option<String>,
    pub fixup: Option<String>,
    pub source_address: Option<String>,
    pub call_home: Option<bool>,
    pub sleep_interval: Option<i32>,
    pub max_sleep_interval: Option<i32>,
    pub listformats: Option<bool>,
    pub list_thumbnails: Option<bool>,
    // pub match_filter
    pub no_color: Option<bool>,
    pub geo_bypass: Option<bool>,
    pub geo_bypass_country: Option<String>,
    pub geo_pypass_ip_block: Option<String>,
    pub external_downloader: Option<String>,
    pub hls_prefer_native: Option<bool>,

    // postprocessor options
    pub prefer_ffmpeg: Option<bool>,
    pub ffmpeg_location: Option<String>,
    pub postproessor_args: Option<Vec<String>>,

    // youtube extractor options
    pub youtube_include_dash_manifest: Option<bool>
}

macro_rules! declare_options_setter {
    ( $attribute:ident, $type:ty ) => {
        concat_idents::concat_idents!{
            set_attribute = set, _, $attribute {
                ///
                /// Sets a youtube-dl option.
                /// See [the youtube-dl documentation](https://github.com/ytdl-org/youtube-dl/blob/master/youtube_dl/YoutubeDL.py#L141) for its meaning"
                ///
                pub fn set_attribute(mut self, $attribute: $type) -> Self {
                    self.$attribute = Some($attribute);
                    self
                }
            }
        }
    }
}

impl YoutubeDlOptions {
    declare_options_setter!(username, String);
    declare_options_setter!(password, String);
    declare_options_setter!(videopassword, String);
    declare_options_setter!(ap_mso, String);
    declare_options_setter!(ap_username, String);
    declare_options_setter!(ap_password, String);
    declare_options_setter!(usenetrc, String);
    declare_options_setter!(verbose, bool);
    declare_options_setter!(quiet, bool);
    declare_options_setter!(no_warnings, bool);
    declare_options_setter!(forceurl, bool);
    declare_options_setter!(forcetitle, bool);
    declare_options_setter!(forceid, bool);
    declare_options_setter!(forcethumbnail, bool);
    declare_options_setter!(forcedescription, bool);
    declare_options_setter!(forcefilename, bool);
    declare_options_setter!(forceduration, bool);
    declare_options_setter!(forcejson, bool);
    declare_options_setter!(dump_single_json, bool);
    declare_options_setter!(simulate, bool);
    declare_options_setter!(format, String);
    declare_options_setter!(outtmpl, String);
    declare_options_setter!(restrictfilenames, bool);
    declare_options_setter!(ignoreerrors, bool);
    declare_options_setter!(force_generic_extractor, bool);
    declare_options_setter!(nooverwites, bool);
    declare_options_setter!(playliststart, i32);
    declare_options_setter!(playlistend, i32);
    declare_options_setter!(playlist_items, Vec<i32>);
    declare_options_setter!(playlistreverse, bool);
    declare_options_setter!(playlistrandom, bool);
    declare_options_setter!(matchtitle, bool);
    declare_options_setter!(rejecttitle, bool);
    declare_options_setter!(logtostderr, bool);
    declare_options_setter!(writedescription, bool);
    declare_options_setter!(writeinfojson, bool);
    declare_options_setter!(writeannotations, bool);
    declare_options_setter!(writethumbnail, bool);
    declare_options_setter!(write_all_thumbnail, bool);
    declare_options_setter!(writesubtitles, bool);
    declare_options_setter!(allsubtitles, bool);
    declare_options_setter!(listsubtitles, bool);
    declare_options_setter!(subtitlesformat, String);
    declare_options_setter!(subtitleslangs, Vec<String>);
    declare_options_setter!(keepvideo, bool);
    declare_options_setter!(skip_download, bool);
    declare_options_setter!(cachedir, String);
    declare_options_setter!(noplaylist, bool);
    declare_options_setter!(age_limit, i32);
    declare_options_setter!(min_views, i32);
    declare_options_setter!(max_view, i32);
    declare_options_setter!(download_archive, String);
    declare_options_setter!(cookiefile, String);
    declare_options_setter!(nocheckcertificate, bool);
    declare_options_setter!(prefer_insecure, bool);
    declare_options_setter!(proxy, String);
    declare_options_setter!(geo_verification_proxy, String);
    declare_options_setter!(socket_timeout, i32);
    declare_options_setter!(bidi_workaround, bool);
    declare_options_setter!(debug_printtraffic, bool);
    declare_options_setter!(include_ads, bool);
    declare_options_setter!(default_search, String);
    declare_options_setter!(encoding, String);
    declare_options_setter!(extract_flat, bool);
    declare_options_setter!(merge_output_format, String);
    declare_options_setter!(fixup, String);
    declare_options_setter!(source_address, String);
    declare_options_setter!(call_home, bool);
    declare_options_setter!(sleep_interval, i32);
    declare_options_setter!(max_sleep_interval, i32);
    declare_options_setter!(listformats, bool);
    declare_options_setter!(list_thumbnails, bool);
    declare_options_setter!(no_color, bool);
    declare_options_setter!(geo_bypass, bool);
    declare_options_setter!(geo_bypass_country, String);
    declare_options_setter!(geo_pypass_ip_block, String);
    declare_options_setter!(external_downloader, String);
    declare_options_setter!(hls_prefer_native, bool);
    declare_options_setter!(prefer_ffmpeg, bool);
    declare_options_setter!(ffmpeg_location, String);
    declare_options_setter!(postproessor_args, Vec<String>);
    declare_options_setter!(youtube_include_dash_manifest, bool);

    pub fn build(self) -> YoutubeDl {
        YoutubeDl {
            options: self
        }
    }
}

///
/// This is the main entrypoint for using the youtube_dl crate.
///
#[derive(Clone, Debug)]
pub struct YoutubeDl {
    options: YoutubeDlOptions
}

impl YoutubeDl {
    ///
    /// Builder-style API to create a YoutubeDl object with options
    ///
    pub fn options() -> YoutubeDlOptions {
        YoutubeDlOptions::default()
    }

    ///
    /// Create a YoutubeDl object with default options
    ///
    pub fn new() -> YoutubeDl {
        YoutubeDl::options().build()
    }

    fn prepare_options(self, py: Python) -> &PyDict {
        let options = PyDict::new(py);

        macro_rules! declare_pydict_setter {
            ( $attribute:ident ) => {
                if self.options.$attribute.is_some() {
                    options.set_item(stringify!($attribute), self.options.$attribute).expect("Failed to set ytdl option");
                }
            }
        }

        // Set options
        declare_pydict_setter!(username);
        declare_pydict_setter!(password);
        declare_pydict_setter!(videopassword);
        declare_pydict_setter!(ap_mso);
        declare_pydict_setter!(ap_username);
        declare_pydict_setter!(ap_password);
        declare_pydict_setter!(usenetrc);
        declare_pydict_setter!(verbose);
        declare_pydict_setter!(quiet);
        declare_pydict_setter!(no_warnings);
        declare_pydict_setter!(forceurl);
        declare_pydict_setter!(forcetitle);
        declare_pydict_setter!(forceid);
        declare_pydict_setter!(forcethumbnail);
        declare_pydict_setter!(forcedescription);
        declare_pydict_setter!(forcefilename);
        declare_pydict_setter!(forceduration);
        declare_pydict_setter!(forcejson);
        declare_pydict_setter!(dump_single_json);
        declare_pydict_setter!(simulate);
        declare_pydict_setter!(format);
        declare_pydict_setter!(outtmpl);
        declare_pydict_setter!(restrictfilenames);
        declare_pydict_setter!(ignoreerrors);
        declare_pydict_setter!(force_generic_extractor);
        declare_pydict_setter!(nooverwites);
        declare_pydict_setter!(playliststart);
        declare_pydict_setter!(playlistend);
        declare_pydict_setter!(playlist_items);
        declare_pydict_setter!(playlistreverse);
        declare_pydict_setter!(playlistrandom);
        declare_pydict_setter!(matchtitle);
        declare_pydict_setter!(rejecttitle);
        declare_pydict_setter!(logtostderr);
        declare_pydict_setter!(writedescription);
        declare_pydict_setter!(writeinfojson);
        declare_pydict_setter!(writeannotations);
        declare_pydict_setter!(writethumbnail);
        declare_pydict_setter!(write_all_thumbnail);
        declare_pydict_setter!(writesubtitles);
        declare_pydict_setter!(allsubtitles);
        declare_pydict_setter!(listsubtitles);
        declare_pydict_setter!(subtitlesformat);
        declare_pydict_setter!(subtitleslangs);
        declare_pydict_setter!(keepvideo);
        declare_pydict_setter!(skip_download);
        declare_pydict_setter!(cachedir);
        declare_pydict_setter!(noplaylist);
        declare_pydict_setter!(age_limit);
        declare_pydict_setter!(min_views);
        declare_pydict_setter!(max_view);
        declare_pydict_setter!(download_archive);
        declare_pydict_setter!(cookiefile);
        declare_pydict_setter!(nocheckcertificate);
        declare_pydict_setter!(prefer_insecure);
        declare_pydict_setter!(proxy);
        declare_pydict_setter!(geo_verification_proxy);
        declare_pydict_setter!(socket_timeout);
        declare_pydict_setter!(bidi_workaround);
        declare_pydict_setter!(debug_printtraffic);
        declare_pydict_setter!(include_ads);
        declare_pydict_setter!(default_search);
        declare_pydict_setter!(encoding);
        declare_pydict_setter!(extract_flat);
        declare_pydict_setter!(merge_output_format);
        declare_pydict_setter!(fixup);
        declare_pydict_setter!(source_address);
        declare_pydict_setter!(call_home);
        declare_pydict_setter!(sleep_interval);
        declare_pydict_setter!(max_sleep_interval);
        declare_pydict_setter!(listformats);
        declare_pydict_setter!(list_thumbnails);
        declare_pydict_setter!(no_color);
        declare_pydict_setter!(geo_bypass);
        declare_pydict_setter!(geo_bypass_country);
        declare_pydict_setter!(geo_pypass_ip_block);
        declare_pydict_setter!(external_downloader);
        declare_pydict_setter!(hls_prefer_native);
        declare_pydict_setter!(prefer_ffmpeg);
        declare_pydict_setter!(ffmpeg_location);
        declare_pydict_setter!(postproessor_args);
        declare_pydict_setter!(youtube_include_dash_manifest);

        return options;
    }

    ///
    /// Download the videos at the provided urls
    ///
    pub fn download(self, urls: &Vec<&str>) -> pyo3::PyResult<i32> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let locals = PyDict::new(py);

        // Set python context
        locals.set_item("urls", urls).expect("Failed to pass url to python");
        locals.set_item("options", self.prepare_options(py)).expect("Failed to pass ytdl options");
        locals.set_item("youtube_dl", py.import("youtube_dl").expect("Failed to import youtube_dl")).expect("Failed to pass ytdl to python");

        let res = py.eval(include_str!("py/download.py"), None, Some(&locals));

        res.and_then(|ret| ret.extract::<i32>())
    }

    ///
    /// Generic function for extracting information using youtube-dl.
    /// If you need to extract information about a youtube video, use extract_video instead.
    /// For playlists, there is extract_playlist.
    ///
    pub fn extract_info(self, url: &str) -> pyo3::PyResult<String> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let locals = PyDict::new(py);

        // Set python context
        locals.set_item("url", url).expect("Failed to pass url to python");
        locals.set_item("options", self.prepare_options(py)).expect("Failed to pass ytdl options");
        locals.set_item("youtube_dl", py.import("youtube_dl").expect("Failed to import youtube_dl")).expect("Failed to pass ytdl to python");
        locals.set_item("json", py.import("json").expect("Failed to import json")).expect("Failed to pass json module to python");

        let ret = py.eval(include_str!("py/extract_info.py"), None, Some(&locals))?;

        Ok(ret.downcast::<PyString>()?.extract::<String>()?)
    }

    ///
    /// Extract information about the video at the provided url
    ///
    pub fn extract_video(self, url: &str) -> PyResult<Video> {
        let json_string = self.extract_info(url)?;
        Ok(serde_json::from_str::<Video>(&json_string).unwrap())
    }

    ///
    /// Extract information about the playlist at the provided url
    ///
    pub fn extract_playlist(self, url: &str) -> PyResult<Playlist> {
        let json_string = self.extract_info(url)?;
        Ok(serde_json::from_str::<Playlist>(&json_string).unwrap())
    }

    ///
    /// Get the list of available extractors
    ///
    pub fn extractors() -> PyResult<Vec<Extractor>> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let locals = PyDict::new(py);
        locals.set_item("youtube_dl", py.import("youtube_dl").expect("Failed to import youtube_dl")).expect("Failed to pass ytdl to python");

        let py_extractors_any = py.eval(include_str!("py/list_extractors.py"), None, Some(&locals))?;

        // Convert python list to vec with properly typed struct
        let py_extractor_list: &PyList = py_extractors_any.downcast::<PyList>()?;
        let extractors: Vec<Extractor> = Extractor::from_py_dict_list(py_extractor_list)?;

        Ok(extractors)
    }
}

#[test]
fn test_default_options() {
    let ytdl = YoutubeDl::new();

    let gil = Python::acquire_gil();
    let pydict = ytdl.prepare_options(gil.python());
    assert!(pydict.get_item("username").is_none());
    assert!(pydict.get_item("password").is_none());
    assert!(pydict.get_item("videopassword").is_none());
    assert!(pydict.get_item("ap_mso").is_none());
    assert!(pydict.get_item("ap_username").is_none());
    assert!(pydict.get_item("ap_password").is_none());
    assert!(pydict.get_item("usenetrc").is_none());
    assert!(pydict.get_item("verbose").is_none());
    assert!(pydict.get_item("quiet").is_none());
    assert!(pydict.get_item("no_warnings").is_none());
    assert!(pydict.get_item("forceurl").is_none());
    assert!(pydict.get_item("forcetitle").is_none());
    assert!(pydict.get_item("forceid").is_none());
    assert!(pydict.get_item("forcethumbnail").is_none());
    assert!(pydict.get_item("forcedescription").is_none());
    assert!(pydict.get_item("forcefilename").is_none());
    assert!(pydict.get_item("forceduration").is_none());
    assert!(pydict.get_item("forcejson").is_none());
    assert!(pydict.get_item("dump_single_json").is_none());
    assert!(pydict.get_item("simulate").is_none());
    assert!(pydict.get_item("format").is_none());
    assert!(pydict.get_item("outtmpl").is_none());
    assert!(pydict.get_item("restrictfilenames").is_none());
    assert!(pydict.get_item("ignoreerrors").is_none());
    assert!(pydict.get_item("force_generic_extractor").is_none());
    assert!(pydict.get_item("nooverwites").is_none());
    assert!(pydict.get_item("playliststart").is_none());
    assert!(pydict.get_item("playlistend").is_none());
    assert!(pydict.get_item("playlist_items").is_none());
    assert!(pydict.get_item("playlistreverse").is_none());
    assert!(pydict.get_item("playlistrandom").is_none());
    assert!(pydict.get_item("matchtitle").is_none());
    assert!(pydict.get_item("rejecttitle").is_none());
    assert!(pydict.get_item("logtostderr").is_none());
    assert!(pydict.get_item("writedescription").is_none());
    assert!(pydict.get_item("writeinfojson").is_none());
    assert!(pydict.get_item("writeannotations").is_none());
    assert!(pydict.get_item("writethumbnail").is_none());
    assert!(pydict.get_item("write_all_thumbnail").is_none());
    assert!(pydict.get_item("writesubtitles").is_none());
    assert!(pydict.get_item("allsubtitles").is_none());
    assert!(pydict.get_item("listsubtitles").is_none());
    assert!(pydict.get_item("subtitlesformat").is_none());
    assert!(pydict.get_item("subtitleslangs").is_none());
    assert!(pydict.get_item("keepvideo").is_none());
    assert!(pydict.get_item("skip_download").is_none());
    assert!(pydict.get_item("cachedir").is_none());
    assert!(pydict.get_item("noplaylist").is_none());
    assert!(pydict.get_item("age_limit").is_none());
    assert!(pydict.get_item("min_views").is_none());
    assert!(pydict.get_item("max_view").is_none());
    assert!(pydict.get_item("download_archive").is_none());
    assert!(pydict.get_item("cookiefile").is_none());
    assert!(pydict.get_item("nocheckcertificate").is_none());
    assert!(pydict.get_item("prefer_insecure").is_none());
    assert!(pydict.get_item("proxy").is_none());
    assert!(pydict.get_item("geo_verification_proxy").is_none());
    assert!(pydict.get_item("socket_timeout").is_none());
    assert!(pydict.get_item("bidi_workaround").is_none());
    assert!(pydict.get_item("debug_printtraffic").is_none());
    assert!(pydict.get_item("include_ads").is_none());
    assert!(pydict.get_item("default_search").is_none());
    assert!(pydict.get_item("encoding").is_none());
    assert!(pydict.get_item("extract_flat").is_none());
    assert!(pydict.get_item("merge_output_format").is_none());
    assert!(pydict.get_item("fixup").is_none());
    assert!(pydict.get_item("source_address").is_none());
    assert!(pydict.get_item("call_home").is_none());
    assert!(pydict.get_item("sleep_interval").is_none());
    assert!(pydict.get_item("max_sleep_interval").is_none());
    assert!(pydict.get_item("listformats").is_none());
    assert!(pydict.get_item("list_thumbnails").is_none());
    assert!(pydict.get_item("no_color").is_none());
    assert!(pydict.get_item("geo_bypass").is_none());
    assert!(pydict.get_item("geo_bypass_country").is_none());
    assert!(pydict.get_item("geo_pypass_ip_block").is_none());
    assert!(pydict.get_item("external_downloader").is_none());
    assert!(pydict.get_item("hls_prefer_native").is_none());
    assert!(pydict.get_item("prefer_ffmpeg").is_none());
    assert!(pydict.get_item("ffmpeg_location").is_none());
    assert!(pydict.get_item("postproessor_args").is_none());
    assert!(pydict.get_item("youtube_include_dash_manifest").is_none());
}

#[test]
fn test_set_options() {
    let ytdl = YoutubeDl::options()
        .set_username("Hello".to_owned())
        .set_password("World".to_owned())
        .build();

    let gil = Python::acquire_gil();
    let pydict = ytdl.prepare_options(gil.python());

    assert_eq!(pydict.get_item("username").unwrap().downcast::<PyString>().unwrap().extract::<String>().unwrap(), "Hello".to_owned());
    assert_eq!(pydict.get_item("password").unwrap().downcast::<PyString>().unwrap().extract::<String>().unwrap(), "World".to_owned());
}
