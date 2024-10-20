use bpaf::Bpaf;

use rodio::Sink;
use rodio::{Decoder, OutputStream};

use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;

use log::{info, warn};

// String constants
const INCORRECT_SPEED: &str = "Speed must be between 0.0 and 2.0";

// Argument guards
fn speed_guard(speed: &f32) -> bool {
    return *speed > 0.0 && *speed < 2.0;
}

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
struct Arguments {
    #[bpaf(flag(true, false))]
    recursive: bool,

    #[bpaf(
        long("speed"),
        short('s'),
        guard(speed_guard, INCORRECT_SPEED),
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
        } else {
            if file_path.extension().and_then(OsStr::to_str) == Some("mp3") {
                // This could be a lossy conversion. Read the docs about this
                file_paths.push(file_path.display().to_string());
            } else {
                warn!(
                    "Found non-mp3 file or directory {}",
                    file_path.display().to_string()
                );
            }
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

fn main() {
    env_logger::init();

    let opts: Arguments = arguments().run();

    // Get an output stream handle to the default physical sound device
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let file_paths = get_file_paths(Path::new(&(opts.audio_path)), opts.recursive);

    // Create a new sink, which allows us to control the audio being played.
    let sink = Sink::try_new(&stream_handle).unwrap();

    // Set the playback speed of the audio based on a command line argument
    sink.set_speed(opts.audio_speed);

    match file_paths {
        Ok(file_paths) => {
            for path in file_paths.iter() {
                info!("Found file: {}", path);

                // Load a sound from a file, using a path relative to Cargo.toml
                let reader = BufReader::new(File::open(path).unwrap());

                // Decode that sound file into a source
                let source = Decoder::new(reader).unwrap();

                // Add the current source to the audio to be played
                sink.append(source);
            }
        }
        Err(_) => todo!(),
    }

    // The audio is played in a separate thread, this call makes the current thread sleep until it is done
    sink.sleep_until_end();
}
