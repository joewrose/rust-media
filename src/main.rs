use bpaf::Bpaf;

use rand::seq::SliceRandom;

use rodio::Sink;
use rodio::{Decoder, OutputStream};

use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use log::{info, warn};

// String constants
const HELP_MESSAGE: &str = r#"
Available commands:
  pause       - Pause audio playback.
  resume      - Resume audio playback.
  skip        - Skip to the next track (if supported).
  speed <n>   - Set playback speed to a specific value (e.g., 'speed 1.5').
  quit        - Stop playback and exit the program.
  help        - Show this help message.
"#;

// Argument guards
fn speed_guard(speed: &f32) -> bool {
    *speed > 0.0 && *speed < 2.0
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
struct Arguments {
    #[bpaf(long("recursive"), short('r'), flag(true, false))]
    recursive: bool,

    #[bpaf(
        long("speed"),
        short('s'),
        guard(speed_guard, "Speed must be between 0.0 and 2.0"),
        fallback(1.0)
    )]
    /// The speed at which you want the audio to play
    pub audio_speed: f32,

    #[bpaf(positional("AUDIO_PATH"))]
    /// The path of the audio file you wish to play
    pub audio_path: String,
}

fn walk_dir(
    dir: &Path,
    file_paths: &mut Vec<String>,
    recursive: bool,
) -> Result<Vec<String>, std::io::Error> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let file_path = entry.path();

        if recursive && file_path.is_dir() {
            walk_dir(&file_path, file_paths, recursive)?;
        } else if file_path.extension().and_then(OsStr::to_str) == Some("mp3") {
            // This could be a lossy conversion. Read the docs about this
            file_paths.push(file_path.display().to_string());
        } else {
            warn!(
                "Found non-mp3 file or directory {}",
                file_path.display().to_string()
            );
        }
    }
    // Should we be converting to a vec here?
    Ok(file_paths.to_vec())
}

fn get_file_paths(dir: &Path, recursive: bool) -> Result<Vec<String>, std::io::Error> {
    match dir.is_dir() {
        // Return a vector containing all audio files in the dir
        true => {
            let mut file_paths: Vec<String> = Vec::new();
            walk_dir(dir, &mut file_paths, recursive)
        }
        // Return a vector of size one
        false => Ok(Vec::from([dir.display().to_string()])),
    }
}

fn set_speed(sink: &rodio::Sink, speed: f32) {
    sink.set_speed(speed.max(0.1).min(2.0));
}

fn set_volume(sink: &rodio::Sink, speed: f32) {
    sink.set_speed(speed.max(0.1).min(2.0));
}

fn main() {
    env_logger::init();

    let opts: Arguments = arguments().run();

    // Get an output stream handle to the default physical sound device
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let file_paths = get_file_paths(Path::new(&(opts.audio_path)), opts.recursive);

    // Create a new sink, which allows us to control the audio being played.
    let sink = Sink::try_new(&stream_handle).unwrap();

    sink.set_speed(opts.audio_speed);

    // Clone the sink so it can be shared across threads
    let sink = Arc::new(Mutex::new(sink));

    // Set the playback speed of the audio based on a command line argument

    match file_paths {
        Ok(mut file_paths) => {
            let mut rng = rand::thread_rng();
            file_paths.shuffle(&mut rng);

            for path in file_paths.iter() {
                info!("Found file: {}", path);

                // Load a sound from a file, using a path relative to Cargo.toml
                let reader = BufReader::new(File::open(path).unwrap());

                // Decode that sound file into a source
                let source = Decoder::new(reader).unwrap();

                // Add the current source to the audio to be played
                {
                    let sink = sink.clone();
                    sink.lock().unwrap().append(source);
                }
            }
        }
        Err(_) => todo!(),
    }

    // Spawn a thread to handle user input
    let sink_clone = sink.clone();
    thread::spawn(move || {
        let mut input = String::new();
        loop {
            // Get user input
            print!("Playing music, type 'help' for command list: ");
            std::io::stdout().flush().unwrap();
            input.clear();
            std::io::stdin().read_line(&mut input).unwrap();

            let sink = sink_clone.lock().unwrap();

            let mut parts = input.split_whitespace();
            if let Some(command) = parts.next() {
                match command {
                    "help" => {
                        println!("{}", HELP_MESSAGE);
                    }
                    "pause" => {
                        sink.pause();
                        println!("Paused.");
                    }
                    "resume" => {
                        sink.play();
                        println!("Resumed.");
                    }
                    "skip" => {
                        sink.skip_one();
                        println!("Skipping track...");
                    }
                    "speed" => {
                        // Check if there's a second argument with the speed value
                        if let Some(speed_str) = parts.next() {
                            if let Ok(speed) = speed_str.parse::<f32>() {
                                set_speed(&sink, speed);
                            } else {
                                println!("Invalid speed value.");
                            }
                        } else {
                            println!("Please provide a speed value.");
                        }
                    }
                    "volume" => {
                        if let Some(speed_str) = parts.next() {
                            if let Ok(speed) = speed_str.parse::<f32>() {
                                set_volume(&sink, speed);
                            } else {
                                println!("Invalid volume value.");
                            }
                        } else {
                            println!("Please provide a volume value.");
                        }
                    }
                    "quit" => {
                        sink.clear();
                        return false;
                    }
                    _ => {
                        println!("Unknown command.");
                    }
                }
            }
        }
    });

    // Keep the main thread alive while audio is playing
    loop {
        thread::sleep(Duration::from_millis(100));
        let sink = sink.lock().unwrap();
        if sink.empty() {
            break;
        }
    }
}
