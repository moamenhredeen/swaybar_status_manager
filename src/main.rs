// use chrono::Local;
use std::{
    io::{BufRead, Read, Write},
    thread,
    time::Duration,
};

use chrono::Local;
use serde::{Deserialize, Serialize};

/// The  header  is  a  JSON object with support for the following
/// properties (only version is required)
#[derive(Serialize, Deserialize)]
struct Header {
    ///The protocol version to use. Currently, this must be 1      
    version: u8,

    /// Whether to receive click event information to standard input
    #[serde(skip_serializing_if = "Option::is_none")]
    click_events: Option<bool>,

    /// The signal that swaybar should send to continue processing
    #[serde(skip_serializing_if = "Option::is_none")]
    const_signal: Option<u32>,

    /// The signal that swaybar should send to stop processing            
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_signal: Option<u32>,
}

impl Header {
    fn new(version: u8) -> Self {
        Self {
            version,
            click_events: Option::None,
            const_signal: Option::None,
            stop_signal: Option::None,
        }
    }
}

/// The body is an infinite array, where each element of the array
/// is a representation of the status line at the  time  that  the
/// element  was  written.  Each element of the array is itself an
/// array of JSON objects, where each object represents a block in
/// the status line. Each block can have the following  properties
/// (only full_text is required)
#[derive(Serialize, Deserialize)]
struct Block {
    /// The text that will be displayed. If missing, the block will be skipped.
    full_text: String,

    /// If given and the text needs to be shortened due to space, this will be displayed instead of full_text  
    #[serde(skip_serializing_if = "Option::is_none")]
    short_text: Option<String>,

    /// A name for the block. This is only used to identify the block for click events. If set, each block should have a unique name and instance pair.
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,

    /// The text color to use in #RRGGBBAA or #RRGGBB notation   
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<String>,

    /// The background color for the block in #RRGGBBAA or #RRGGBB notation  
    #[serde(skip_serializing_if = "Option::is_none")]
    background: Option<String>,

    /// The border color for the block in #RRGGBBAA or #RRGGBB notation
    #[serde(skip_serializing_if = "Option::is_none")]
    border: Option<String>,

    /// The height in pixels of the top border. The default is 1
    #[serde(skip_serializing_if = "Option::is_none")]
    border_top: Option<u8>,

    /// The width in pixels of the right border. The default is 1
    #[serde(skip_serializing_if = "Option::is_none")]
    border_right: Option<u8>,

    /// The height in pixels of the bottom border. The default is 1
    #[serde(skip_serializing_if = "Option::is_none")]
    border_bottom: Option<u8>,

    /// The width in pixels of the left border. The default is 1
    #[serde(skip_serializing_if = "Option::is_none")]
    border_left: Option<u8>,

    /// The minimum width to use for the block. This can either be given in pixels
    /// or a string can be given to allow for it to be calculated based on the width of the string.
    #[serde(skip_serializing_if = "Option::is_none")]
    min_width: Option<u32>,

    /// If the text does not span the full width of the block,
    /// this specifies how the text should be aligned inside of the block. This can be left (default),
    /// right, or center.
    #[serde(skip_serializing_if = "Option::is_none")]
    align: Option<String>,

    /// The instance of the name for the block. This is only used to identify the block for click events.
    /// If set, each block should have a unique name and instance pair.
    #[serde(skip_serializing_if = "Option::is_none")]
    instance: Option<String>,

    /// Whether the block should be displayed as urgent.
    /// Currently swaybar utilizes the colors set in the sway config for urgent workspace buttons.
    /// See sway-bar(5) for more information on bar color con‐ figuration.  
    #[serde(skip_serializing_if = "Option::is_none")]
    urgent: Option<bool>,

    /// Whether the bar separator should be drawn after the block.
    /// See sway-bar(5) for more information on how to set the separator text.
    #[serde(skip_serializing_if = "Option::is_none")]
    seperator: Option<bool>,

    /// The amount of pixels to leave blank after the block.
    /// The separator text will be displayed cen‐ tered in this gap. The default is 9 pixels.
    #[serde(skip_serializing_if = "Option::is_none")]
    seperator_block_width: Option<bool>,

    /// The type of markup to use when parsing the text for the block.
    /// This can either be pango or none (default).  
    #[serde(skip_serializing_if = "Option::is_none")]
    markup: Option<String>,
}

impl Block {
    fn new(full_text: String) -> Self {
        Self {
            full_text,
            short_text: Option::None,
            name: Option::None,
            color: Option::None,
            background: Option::None,
            border: Option::None,
            border_top: Option::None,
            border_right: Option::None,
            border_bottom: Option::None,
            border_left: Option::None,
            min_width: Option::None,
            align: Option::None,
            instance: Option::None,
            urgent: Option::None,
            seperator: Option::None,
            seperator_block_width: Option::None,
            markup: Option::None,
        }
    }

    fn color(&mut self, color: String) {
        self.color = Some(color);
    }

    fn background(&mut self, background: String) {
        self.background = Some(background);
    }

    fn with_seperator(&mut self) {
        self.seperator = Some(true);
    }
}

/// If requested in the header, swaybar will write a JSON object, that can be read from standard  in,  when  the
/// user clicks on a block. The event object will have the following properties:
#[derive(Serialize, Deserialize)]
struct ClientEvent {
    name: String,
    instance: String,
    x: u32,
    y: u32,
    button: u32,
    event: u32,
    relative_x: u32,
    relative_y: u32,
    width: u32,
    height: u32,
}

fn main() {
    let mut stdin = std::io::stdin().lock();
    let mut stdout = std::io::stdout().lock();

    let header = serde_json::to_string(&Header::new(1)).unwrap();
    write!(stdout, "{}\n[", header).unwrap();
    stdout.flush().unwrap();

    loop {
        let mut block = Block::new(Local::now().format("%H:%M:%S  %Y.%m.%d").to_string());
        block.with_seperator();
        let body = serde_json::to_string(&block).unwrap();
        write!(stdout, "[{}],", body).unwrap();
        stdout.flush().unwrap();
        thread::sleep(Duration::from_secs(1));
        let mut buf: Vec<u8> = Vec::new();
        stdin.read_until(b'}', &mut buf).unwrap();
        stdout.flush().unwrap();
    }
}
