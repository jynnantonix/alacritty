// Copyright 2016 Joe Wilm, The Alacritty Project Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Deserializer};

mod bindings;
mod colors;
mod debug;
mod font;
mod monitor;
mod mouse;
mod scrolling;
#[cfg(test)]
mod test;
mod visual_bell;
mod window;

use crate::ansi::CursorStyle;
use crate::input::{Binding, KeyBinding, MouseBinding};

pub use crate::config::bindings::Key;
pub use crate::config::colors::Colors;
pub use crate::config::debug::Debug;
pub use crate::config::font::{Font, FontDescription};
pub use crate::config::monitor::Monitor;
pub use crate::config::mouse::{ClickHandler, Mouse};
pub use crate::config::scrolling::Scrolling;
pub use crate::config::visual_bell::{VisualBellAnimation, VisualBellConfig};
pub use crate::config::window::{Decorations, Dimensions, StartupMode, WindowConfig};

pub static DEFAULT_ALACRITTY_CONFIG: &'static str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../alacritty.yml"));
const MAX_SCROLLBACK_LINES: u32 = 100_000;

/// Top-level config type
#[derive(Debug, PartialEq, Deserialize)]
pub struct Config {
    /// Pixel padding
    #[serde(default, deserialize_with = "failure_default")]
    pub padding: Option<Delta<u8>>,

    /// TERM env variable
    #[serde(default, deserialize_with = "failure_default")]
    pub env: HashMap<String, String>,

    /// Font configuration
    #[serde(default, deserialize_with = "failure_default")]
    pub font: Font,

    /// Should draw bold text with brighter colors instead of bold font
    #[serde(default, deserialize_with = "failure_default")]
    draw_bold_text_with_bright_colors: DefaultTrueBool,

    #[serde(default, deserialize_with = "failure_default")]
    pub colors: Colors,

    /// Background opacity from 0.0 to 1.0
    #[serde(default, deserialize_with = "failure_default")]
    background_opacity: Alpha,

    /// Window configuration
    #[serde(default, deserialize_with = "failure_default")]
    pub window: WindowConfig,

    /// Keybindings
    #[serde(default = "default_key_bindings", deserialize_with = "deserialize_key_bindings")]
    pub key_bindings: Vec<KeyBinding>,

    /// Bindings for the mouse
    #[serde(default = "default_mouse_bindings", deserialize_with = "deserialize_mouse_bindings")]
    pub mouse_bindings: Vec<MouseBinding>,

    #[serde(default, deserialize_with = "failure_default")]
    pub selection: Selection,

    #[serde(default, deserialize_with = "failure_default")]
    pub mouse: Mouse,

    /// Path to a shell program to run on startup
    #[serde(default, deserialize_with = "failure_default")]
    pub shell: Option<Shell<'static>>,

    /// Path where config was loaded from
    #[serde(default, deserialize_with = "failure_default")]
    pub config_path: Option<PathBuf>,

    /// Visual bell configuration
    #[serde(default, deserialize_with = "failure_default")]
    pub visual_bell: VisualBellConfig,

    /// Use dynamic title
    #[serde(default, deserialize_with = "failure_default")]
    dynamic_title: DefaultTrueBool,

    /// Live config reload
    #[serde(default, deserialize_with = "failure_default")]
    live_config_reload: DefaultTrueBool,

    /// Number of spaces in one tab
    #[serde(default, deserialize_with = "failure_default")]
    tabspaces: Tabspaces,

    /// How much scrolling history to keep
    #[serde(default, deserialize_with = "failure_default")]
    pub scrolling: Scrolling,

    /// Cursor configuration
    #[serde(default, deserialize_with = "failure_default")]
    pub cursor: Cursor,

    /// Enable experimental conpty backend instead of using winpty.
    /// Will only take effect on Windows 10 Oct 2018 and later.
    #[cfg(windows)]
    #[serde(default, deserialize_with = "failure_default")]
    pub enable_experimental_conpty_backend: bool,

    /// Send escape sequences using the alt key.
    #[serde(default, deserialize_with = "failure_default")]
    alt_send_esc: DefaultTrueBool,

    /// Shell startup directory
    #[serde(default, deserialize_with = "failure_default")]
    working_directory: WorkingDirectory,

    /// Forward stdin to `self.shell`
    #[serde(default, deserialize_with = "failure_default")]
    pub inherit_stdin: bool,

    /// Debug options
    #[serde(default, deserialize_with = "failure_default")]
    pub debug: Debug,

    // TODO: DEPRECATED
    #[serde(default, deserialize_with = "failure_default")]
    pub render_timer: Option<bool>,

    // TODO: DEPRECATED
    #[serde(default, deserialize_with = "failure_default")]
    pub persistent_logging: Option<bool>,
}

impl Default for Config {
    fn default() -> Self {
        serde_yaml::from_str(DEFAULT_ALACRITTY_CONFIG).expect("default config is invalid")
    }
}

impl Config {
    pub fn tabspaces(&self) -> usize {
        self.tabspaces.0
    }

    #[inline]
    pub fn draw_bold_text_with_bright_colors(&self) -> bool {
        self.draw_bold_text_with_bright_colors.0
    }

    /// Should show render timer
    #[inline]
    pub fn render_timer(&self) -> bool {
        self.render_timer.unwrap_or(self.debug.render_timer)
    }

    /// Live config reload
    #[inline]
    pub fn live_config_reload(&self) -> bool {
        self.live_config_reload.0
    }

    #[inline]
    pub fn set_live_config_reload(&mut self, live_config_reload: bool) {
        self.live_config_reload.0 = live_config_reload;
    }

    #[inline]
    pub fn dynamic_title(&self) -> bool {
        self.dynamic_title.0
    }

    #[inline]
    pub fn set_dynamic_title(&mut self, dynamic_title: bool) {
        self.dynamic_title.0 = dynamic_title;
    }

    /// Send escape sequences using the alt key
    #[inline]
    pub fn alt_send_esc(&self) -> bool {
        self.alt_send_esc.0
    }

    /// Keep the log file after quitting Alacritty
    #[inline]
    pub fn persistent_logging(&self) -> bool {
        self.persistent_logging.unwrap_or(self.debug.persistent_logging)
    }

    #[inline]
    pub fn background_opacity(&self) -> f32 {
        self.background_opacity.0
    }

    #[inline]
    pub fn working_directory(&self) -> &Option<PathBuf> {
        &self.working_directory.0
    }

    #[inline]
    pub fn set_working_directory(&mut self, working_directory: Option<PathBuf>) {
        self.working_directory.0 = working_directory;
    }
}

#[derive(Default, Debug, PartialEq, Eq)]
struct WorkingDirectory(Option<PathBuf>);

impl<'de> Deserialize<'de> for WorkingDirectory {
    fn deserialize<D>(deserializer: D) -> Result<WorkingDirectory, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_yaml::Value::deserialize(deserializer)?;

        // Accept `None` to use the default path
        if value.as_str().filter(|v| v.to_lowercase() == "none").is_some() {
            return Ok(WorkingDirectory(None));
        }

        Ok(match PathBuf::deserialize(value) {
            Ok(path) => WorkingDirectory(Some(path)),
            Err(err) => {
                error!("Problem with config: {}; using None", err);
                WorkingDirectory(None)
            },
        })
    }
}

fn default_key_bindings() -> Vec<KeyBinding> {
    bindings::default_key_bindings()
}

fn default_mouse_bindings() -> Vec<MouseBinding> {
    bindings::default_mouse_bindings()
}

fn deserialize_key_bindings<'a, D>(deserializer: D) -> Result<Vec<KeyBinding>, D::Error>
where
    D: Deserializer<'a>,
{
    deserialize_bindings(deserializer, bindings::default_key_bindings())
}

fn deserialize_mouse_bindings<'a, D>(deserializer: D) -> Result<Vec<MouseBinding>, D::Error>
where
    D: Deserializer<'a>,
{
    deserialize_bindings(deserializer, bindings::default_mouse_bindings())
}

fn deserialize_bindings<'a, D, T>(
    deserializer: D,
    mut default: Vec<Binding<T>>,
) -> Result<Vec<Binding<T>>, D::Error>
where
    D: Deserializer<'a>,
    T: Copy + Eq + std::hash::Hash + std::fmt::Debug,
    Binding<T>: Deserialize<'a>,
{
    let mut bindings: Vec<Binding<T>> = failure_default(deserializer)?;

    for binding in bindings.iter() {
        default.retain(|b| !b.triggers_match(binding));
    }

    bindings.extend(default);

    Ok(bindings)
}

#[serde(default)]
#[derive(Deserialize, Default, Clone, Debug, PartialEq, Eq)]
pub struct Selection {
    #[serde(deserialize_with = "failure_default")]
    semantic_escape_chars: EscapeChars,
    #[serde(deserialize_with = "failure_default")]
    pub save_to_clipboard: bool,
}

impl Selection {
    pub fn semantic_escape_chars(&self) -> &str {
        &self.semantic_escape_chars.0
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct EscapeChars(String);

impl Default for EscapeChars {
    fn default() -> Self {
        EscapeChars(String::from(",│`|:\"' ()[]{}<>"))
    }
}

#[serde(default)]
#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct Cursor {
    #[serde(deserialize_with = "failure_default")]
    pub style: CursorStyle,
    #[serde(deserialize_with = "failure_default")]
    unfocused_hollow: DefaultTrueBool,
}

impl Default for Cursor {
    fn default() -> Self {
        Self { style: Default::default(), unfocused_hollow: Default::default() }
    }
}

impl Cursor {
    pub fn unfocused_hollow(self) -> bool {
        self.unfocused_hollow.0
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Shell<'a> {
    pub program: Cow<'a, str>,

    #[serde(default, deserialize_with = "failure_default")]
    pub args: Vec<String>,
}

impl<'a> Shell<'a> {
    pub fn new<S>(program: S) -> Shell<'a>
    where
        S: Into<Cow<'a, str>>,
    {
        Shell { program: program.into(), args: Vec::new() }
    }

    pub fn new_with_args<S>(program: S, args: Vec<String>) -> Shell<'a>
    where
        S: Into<Cow<'a, str>>,
    {
        Shell { program: program.into(), args }
    }
}

/// A delta for a point in a 2 dimensional plane
#[serde(default, bound(deserialize = "T: Deserialize<'de> + Default"))]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
pub struct Delta<T: Default + PartialEq + Eq> {
    /// Horizontal change
    #[serde(deserialize_with = "failure_default")]
    pub x: T,
    /// Vertical change
    #[serde(deserialize_with = "failure_default")]
    pub y: T,
}

/// Wrapper around f32 that represents an alpha value between 0.0 and 1.0
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Alpha(f32);

impl Alpha {
    pub fn new(value: f32) -> Self {
        Alpha(if value < 0.0 {
            0.0
        } else if value > 1.0 {
            1.0
        } else {
            value
        })
    }
}

impl Default for Alpha {
    fn default() -> Self {
        Alpha(1.0)
    }
}

impl<'a> Deserialize<'a> for Alpha {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        Ok(Alpha::new(f32::deserialize(deserializer)?))
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
struct Tabspaces(usize);

impl Default for Tabspaces {
    fn default() -> Self {
        Tabspaces(8)
    }
}

#[derive(Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
struct DefaultTrueBool(bool);

impl Default for DefaultTrueBool {
    fn default() -> Self {
        DefaultTrueBool(true)
    }
}

pub fn failure_default<'a, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'a>,
    T: Deserialize<'a> + Default,
{
    let value = serde_yaml::Value::deserialize(deserializer)?;
    match T::deserialize(value) {
        Ok(value) => Ok(value),
        Err(err) => {
            error!("Problem with config: {}; using default value", err);
            Ok(T::default())
        },
    }
}
