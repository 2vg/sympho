pub mod model;
pub use crate::model::*;

use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;
use youtube_dl_pyo3::YoutubeDl as YtDl;
use youtube_dl_pyo3::YoutubeDlOptions as YtDlOptions;

/// Data returned by `YoutubeDl::run`. Output can either be a single video or a playlist of videos.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum YoutubeDlOutput {
    /// Playlist result
    Playlist(Box<Playlist>),
    /// Single video result
    SingleVideo(Box<SingleVideo>),
}

impl YoutubeDlOutput {
    #[cfg(test)]
    fn to_single_video(self) -> SingleVideo {
        match self {
            YoutubeDlOutput::SingleVideo(video) => *video,
            _ => panic!("this is a playlist, not a single video"),
        }
    }
    #[cfg(test)]
    fn to_playlist(self) -> Playlist {
        match self {
            YoutubeDlOutput::Playlist(playlist) => *playlist,
            _ => panic!("this is a playlist, not a single video"),
        }
    }
}

/// Errors that can occur during executing `youtube-dl` or during parsing the output.
#[derive(Debug)]
pub enum Error {
    /// I/O error
    Io(std::io::Error),

    /// Error parsing JSON
    Json(serde_json::Error),

    /// `youtube-dl` returned a non-zero exit code
    ExitCode {
        /// Exit code
        code: i32,
        /// Standard error of youtube-dl
        stderr: String,
    },

    /// Process-level timeout expired.
    ProcessTimeout,

    /// Pyo3 Error.
    Pyo3Error,
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "io error: {}", err),
            Self::Json(err) => write!(f, "json error: {}", err),
            Self::ExitCode { code, stderr } => {
                write!(f, "non-zero exit code: {}, stderr: {}", code, stderr)
            }
            Self::ProcessTimeout => write!(f, "process timed out"),
            Self::Pyo3Error => write!(f, "pyo3 error raised"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::Json(err) => Some(err),
            Self::ExitCode { .. } => None,
            Self::ProcessTimeout => None,
            Self::Pyo3Error => None,
        }
    }
}

/// The search options currently supported by youtube-dl, and a custom option to allow
/// specifying custom options, in case this library is outdated.
#[derive(Clone, Debug)]
pub enum SearchType {
    /// Search on youtube.com
    Youtube,
    /// Search with yahoo.com's video search
    Yahoo,
    /// Search with Google's video search
    Google,
    /// Search on SoundCloud
    SoundCloud,
    /// Allows to specify a custom search type, for forwards compatibility purposes.
    Custom(String),
}

impl fmt::Display for SearchType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SearchType::Yahoo => write!(f, "yvsearch"),
            SearchType::Youtube => write!(f, "ytsearch"),
            SearchType::Google => write!(f, "gvsearch"),
            SearchType::SoundCloud => write!(f, "scsearch"),
            SearchType::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// Specifies where to search, how many results to fetch and the query. The count
/// defaults to 1, but can be changed with the `with_count` method.
#[derive(Clone, Debug)]
pub struct SearchOptions {
    search_type: SearchType,
    count: usize,
    query: String,
}

impl SearchOptions {
    /// Search on youtube.com
    pub fn youtube(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            search_type: SearchType::Youtube,
            count: 1,
        }
    }
    /// Search with Google's video search
    pub fn google(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            search_type: SearchType::Google,
            count: 1,
        }
    }
    /// Search with yahoo.com's video search
    pub fn yahoo(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            search_type: SearchType::Yahoo,
            count: 1,
        }
    }
    /// Search on SoundCloud
    pub fn soundcloud(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            search_type: SearchType::SoundCloud,
            count: 1,
        }
    }
    /// Search with a custom search provider (in case this library falls behind the feature set of youtube-dl)
    pub fn custom(search_type: impl Into<String>, query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            search_type: SearchType::Custom(search_type.into()),
            count: 1,
        }
    }
    /// Set the count for how many videos at most to retrieve from the search.
    pub fn with_count(self, count: usize) -> Self {
        Self {
            search_type: self.search_type,
            query: self.query,
            count,
        }
    }
}

impl fmt::Display for SearchOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}:{}", self.search_type, self.count, self.query)
    }
}

/// A builder to create a `youtube-dl` command to execute.
#[derive(Clone, Debug)]
pub struct YoutubeDl {
    inner_options: YtDlOptions,
    youtube_dl_path: Option<PathBuf>,
    _format: Option<String>,
    _flat_playlist: bool,
    _socket_timeout: Option<String>,
    _all_formats: bool,
    _auth: Option<(String, String)>,
    _cookies: Option<String>,
    _user_agent: Option<String>,
    _referer: Option<String>,
    url: String,
    _process_timeout: Option<Duration>,
    _extract_audio: bool,
    _extra_args: Vec<String>,
}

impl YoutubeDl {
    /// Create a new builder.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            inner_options: YtDl::options(),
            url: url.into(),
            youtube_dl_path: None,
            _format: None,
            _flat_playlist: false,
            _socket_timeout: None,
            _all_formats: false,
            _auth: None,
            _cookies: None,
            _user_agent: None,
            _referer: None,
            _process_timeout: None,
            _extract_audio: false,
            _extra_args: Vec::new(),
        }
    }

    /// Performs a search with the given search options.
    pub fn search_for(options: &SearchOptions) -> Self {
        Self::new(options.to_string())
    }

    /// Set the path to the `youtube-dl` executable.
    pub fn youtube_dl_path<P: AsRef<Path>>(&mut self, youtube_dl_path: P) -> &mut Self {
        self.youtube_dl_path = Some(youtube_dl_path.as_ref().to_owned());
        self
    }

    /// Set the `-f` command line option.
    pub fn format<S: Into<String>>(&mut self, format: S) -> &mut Self {
        self.inner_options = self.inner_options.clone().set_format(format.into());
        self
    }

    /// Set the `--flat-playlist` command line flag.
    pub fn flat_playlist(&mut self, flat_playlist: bool) -> &mut Self {
        self.inner_options = self.inner_options.clone().set_extract_flat(flat_playlist);
        self
    }

    /// Set the `--socket-timeout` command line flag.
    pub fn socket_timeout<I: Into<i32>>(&mut self, socket_timeout: I) -> &mut Self {
        self.inner_options = self
            .inner_options
            .clone()
            .set_socket_timeout(socket_timeout.into());
        self
    }

    /*
    /// Set the `--user-agent` command line flag.
    pub fn user_agent<S: Into<String>>(&mut self, user_agent: S) -> &mut Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Set the `--referer` command line flag.
    pub fn referer<S: Into<String>>(&mut self, referer: S) -> &mut Self {
        self.referer = Some(referer.into());
        self
    }

    /// Set the `--all-formats` command line flag.
    pub fn all_formats(&mut self, all_formats: bool) -> &mut Self {
        self.all_formats = all_formats;
        self
    }
    */

    /// Set the `-u` and `-p` command line flags.
    pub fn auth<S: Into<String>>(&mut self, username: S, password: S) -> &mut Self {
        self.inner_options = self
            .inner_options
            .clone()
            .set_username(username.into())
            .set_password(password.into());
        self
    }

    /// Specify a file with cookies in Netscape cookie format.
    pub fn cookies<S: Into<String>>(&mut self, cookie_path: S) -> &mut Self {
        self.inner_options = self
            .inner_options
            .clone()
            .set_cookiefile(cookie_path.into());
        self
    }

    /*
    /// Set a process-level timeout for youtube-dl. (this controls the maximum overall duration
    /// the process may take, when it times out, `Error::ProcessTimeout` is returned)
    pub fn process_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.process_timeout = Some(timeout);
        self
    }

    /// Set the `--extract-audio` command line flag.
    pub fn extract_audio(&mut self, extract_audio: bool) -> &mut Self {
        self.extract_audio = extract_audio;
        self
    }

    /// Add an additional custom CLI argument.
    ///
    /// This allows specifying arguments that are not covered by other
    /// configuration methods.
    pub fn extra_arg<S: Into<String>>(&mut self, arg: S) -> &mut Self {
        self.extra_args.push(arg.into());
        self
    }

    fn path(&self) -> &Path {
        match &self.youtube_dl_path {
            Some(path) => path,
            None => Path::new("youtube-dl"),
        }
    }

    fn process_args(&self) -> Vec<&str> {
        let mut args = vec![];
        if let Some(format) = &self.format {
            args.push("-f");
            args.push(format);
        }

        if self.flat_playlist {
            args.push("--flat-playlist");
        }

        if let Some(timeout) = &self.socket_timeout {
            args.push("--socket-timeout");
            args.push(timeout);
        }

        if self.all_formats {
            args.push("--all-formats");
        }

        if let Some((user, password)) = &self.auth {
            args.push("-u");
            args.push(user);
            args.push("-p");
            args.push(password);
        }

        if let Some(cookie_path) = &self.cookies {
            args.push("--cookies");
            args.push(cookie_path);
        }

        if let Some(user_agent) = &self.user_agent {
            args.push("--user-agent");
            args.push(user_agent);
        }

        if let Some(referer) = &self.referer {
            args.push("--referer");
            args.push(referer);
        }

        if self.extract_audio {
            args.push("--extract-audio");
        }

        for extra_arg in &self.extra_args {
            args.push(extra_arg);
        }

        args.push("-J");
        args.push(&self.url);
        log::debug!("youtube-dl arguments: {:?}", args);

        args
    }
    */

    /// Run youtube-dl with the arguments specified through the builder.
    pub fn run(&self) -> Result<YoutubeDlOutput, Error> {
        use serde_json::{json, Value};

        let ytdl = self.inner_options.clone().set_quiet(true).build();

        let res = if let Ok(r) = ytdl.extract_info(&self.url) {
            r
        } else {
            return Err(Error::Pyo3Error);
        };

        let json_res: Value = serde_json::from_str(&res)?;

        let is_playlist = json_res["_type"] == json!("playlist");
        if is_playlist {
            let playlist: Playlist = serde_json::from_value(json_res)?;
            Ok(YoutubeDlOutput::Playlist(Box::new(playlist)))
        } else {
            let video: SingleVideo = serde_json::from_value(json_res)?;
            Ok(YoutubeDlOutput::SingleVideo(Box::new(video)))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{SearchOptions, YoutubeDl};

    #[test]
    fn test_youtube_url() {
        let output = YoutubeDl::new("https://www.youtube.com/watch?v=7XGyWcuYVrg")
            .socket_timeout(15)
            .run()
            .unwrap()
            .to_single_video();
        assert_eq!(output.id, "7XGyWcuYVrg");
    }

    #[test]
    fn test_with_timeout() {
        let output = YoutubeDl::new("https://www.youtube.com/watch?v=7XGyWcuYVrg")
            .socket_timeout(15)
            .run()
            .unwrap()
            .to_single_video();
        assert_eq!(output.id, "7XGyWcuYVrg");
    }

    #[test]
    fn test_unknown_url() {
        YoutubeDl::new("https://www.rust-lang.org")
            .socket_timeout(15)
            .run()
            .unwrap_err();
    }

    #[test]
    fn test_search() {
        let output = YoutubeDl::search_for(&SearchOptions::youtube("Never Gonna Give You Up"))
            .socket_timeout(15)
            .run()
            .unwrap()
            .to_playlist();
        assert_eq!(output.entries.unwrap().first().unwrap().id, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_video_with_season() {
        let output = YoutubeDl::new("https://youtube.com/watch?v=sAD1nayZ9dk")
            .run()
            .unwrap()
            .to_single_video();

        assert_eq!(output.season_number, Some(2));
    }

    #[test]
    fn correct_format_codec_parsing() {
        let output = YoutubeDl::new("https://www.youtube.com/watch?v=WhWc3b3KhnY")
            .run()
            .unwrap()
            .to_single_video();

        let mut none_counter = 0;
        for format in output.formats.unwrap() {
            assert_ne!(Some("none".to_string()), format.acodec);
            assert_ne!(Some("none".to_string()), format.vcodec);
            if format.acodec.is_none() || format.vcodec.is_none() {
                none_counter += 1;
            }
        }
        assert!(none_counter > 0);
    }
}
