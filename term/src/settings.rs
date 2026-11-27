//! The options, the config file and which wins (D125 items 1, 5 and 6; docs/TERMINAL.md, "The
//! command, the menu and the config").
//!
//! SPDX-License-Identifier: MIT
use crate::audio::Backend;
use crate::host::Mode;
use std::path::{Path, PathBuf};

/// The instruments the menu lists, in its order (D125 item 4): a name and a few words.
pub const INSTRUMENTS: [(&str, &str); 1] = [("rhythm", "a sixteen-step drum machine")];

/// The key mode as asked for: `None` is auto.
pub type Keys = Option<Mode>;

/// What the command line said. `None` is "not given".
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Args {
    /// The instrument named, as its place in `INSTRUMENTS`.
    pub instrument: Option<usize>,
    pub keys: Option<Keys>,
    pub pocket: bool,
    pub no_colour: bool,
    pub audio: Option<Backend>,
    pub beat: Option<PathBuf>,
    pub reduced: bool,
    pub help: bool,
    pub version: bool,
}

fn keys_of(v: &str) -> Option<Keys> {
    match v {
        "auto" => Some(None),
        "hold" => Some(Some(Mode::Hold)),
        "tap" => Some(Some(Mode::Tap)),
        _ => None,
    }
}

fn audio_of(v: &str) -> Option<Backend> {
    match v {
        "default" | "alsa" => Some(Backend::Default),
        "pipewire" => Some(Backend::PipeWire),
        "none" => Some(Backend::None),
        _ => None,
    }
}

/// The command line, without the program's name. Options go before or after the instrument.
pub fn parse_args(args: &[String]) -> Result<Args, String> {
    let mut a = Args::default();
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        let mut value = |name: &str| it.next().cloned().ok_or_else(|| format!("{name} needs a value"));
        match arg.as_str() {
            "-h" | "--help" => a.help = true,
            "-V" | "--version" => a.version = true,
            "--keys" => {
                let v = value("--keys")?;
                a.keys = Some(keys_of(&v).ok_or_else(|| format!("--keys {v}: auto, hold or tap"))?);
            }
            "--pocket" => a.pocket = true,
            "--no-color" | "--no-colour" => a.no_colour = true,
            "--reduced-motion" => a.reduced = true,
            "--audio" => {
                let v = value("--audio")?;
                a.audio = Some(audio_of(&v).ok_or_else(|| format!("--audio {v}: default, pipewire or none"))?);
            }
            "--beat" => a.beat = Some(PathBuf::from(value("--beat")?)),
            s if s.starts_with('-') => return Err(format!("unknown option {s} (see --help)")),
            s => match INSTRUMENTS.iter().position(|(n, _)| *n == s) {
                Some(_) if a.instrument.is_some() => return Err(format!("one instrument at a time: {s}")),
                Some(i) => a.instrument = Some(i),
                None => return Err(format!("no instrument called {s} (there is: rhythm)")),
            },
        }
    }
    Ok(a)
}

/// What the config file said. `None` is "not in the file".
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Config {
    pub keys: Option<Keys>,
    pub colour: Option<bool>,
    pub audio: Option<Backend>,
    pub reduced: Option<bool>,
}

/// `$XDG_CONFIG_HOME/pocket-instruments/config.toml`, or `~/.config/…` (D125 item 5).
pub fn config_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").filter(|v| !v.is_empty()).map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("pocket-instruments").join("config.toml"))
}

/// The file at `path`; a missing file is an empty config, not an error.
pub fn read_config(path: &Path) -> Result<Config, String> {
    match std::fs::read_to_string(path) {
        Ok(text) => parse_config(&text, &path.display().to_string()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

/// A subset of TOML (D125 item 5): `key = value` a line, strings in double quotes, `true` or
/// `false`, `#` comments and blank lines. Anything else is refused with the file and the line.
pub fn parse_config(text: &str, file: &str) -> Result<Config, String> {
    let mut c = Config::default();
    for (i, raw) in text.lines().enumerate() {
        let at = |what: String| format!("{file}:{}: {what}", i + 1);
        let line = strip_comment(raw).trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') {
            return Err(at("tables are not read here; one key = value a line".into()));
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(at(format!("not key = value: {line}")));
        };
        let (key, value) = (key.trim(), value.trim());
        let string = || -> Result<&str, String> {
            value
                .strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .filter(|v| !v.contains('"') && !v.contains('\\'))
                .ok_or_else(|| at(format!("{key} takes a string in double quotes, as \"{}\"", example(key))))
        };
        let boolean = || -> Result<bool, String> {
            match value {
                "true" => Ok(true),
                "false" => Ok(false),
                _ => Err(at(format!("{key} takes true or false"))),
            }
        };
        match key {
            "keys" => {
                let v = string()?;
                c.keys = Some(keys_of(v).ok_or_else(|| at(format!("keys = \"{v}\": \"auto\", \"hold\" or \"tap\"")))?);
            }
            "colour" | "color" => c.colour = Some(boolean()?),
            "audio" => {
                let v = string()?;
                c.audio = Some(audio_of(v).ok_or_else(|| at(format!("audio = \"{v}\": \"default\", \"pipewire\" or \"none\"")))?);
            }
            "reduced_motion" => c.reduced = Some(boolean()?),
            other => return Err(at(format!("unknown key {other} (keys, colour, audio, reduced_motion)"))),
        }
    }
    Ok(c)
}

fn example(key: &str) -> &'static str {
    if key == "audio" { "default" } else { "auto" }
}

/// A `#` outside a string starts a comment.
fn strip_comment(line: &str) -> &str {
    let mut quoted = false;
    for (i, ch) in line.char_indices() {
        match ch {
            '"' => quoted = !quoted,
            '#' if !quoted => return &line[..i],
            _ => {}
        }
    }
    line
}

/// Where a setting came from, for the intro page (D125 item 2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum From {
    Option,
    Config,
    NoColor,
    Default,
}

impl From {
    pub fn word(self) -> &'static str {
        match self {
            From::Option => "option",
            From::Config => "config",
            From::NoColor => "NO_COLOR",
            From::Default => "default",
        }
    }
}

/// What an instrument plays with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub keys: (Keys, From),
    pub mono: (bool, From),
    pub audio: (Backend, From),
    pub reduced: (bool, From),
    pub pocket: bool,
    pub beat: Option<PathBuf>,
}

/// The order of D125 item 6: the command line, then the file, then `NO_COLOR`, then the default.
pub fn resolve(a: &Args, c: &Config, no_color_env: bool) -> Settings {
    fn pick<T>(opt: Option<T>, conf: Option<T>, default: T) -> (T, From) {
        match (opt, conf) {
            (Some(v), _) => (v, From::Option),
            (None, Some(v)) => (v, From::Config),
            (None, None) => (default, From::Default),
        }
    }
    let mono = if a.no_colour {
        (true, From::Option)
    } else if let Some(colour) = c.colour {
        (!colour, From::Config)
    } else if no_color_env {
        (true, From::NoColor)
    } else {
        (false, From::Default)
    };
    Settings {
        keys: pick(a.keys, c.keys, None),
        mono,
        audio: pick(a.audio, c.audio, Backend::Default),
        reduced: pick(a.reduced.then_some(true), c.reduced, false),
        pocket: a.pocket,
        beat: a.beat.clone(),
    }
}
