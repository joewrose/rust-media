pub mod interface;
pub mod search;

use bpaf::Bpaf;

use rand::seq::SliceRandom;

use rodio::Sink;
use rodio::{Decoder, OutputStream};

use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use log::info;

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

fn main() {
    env_logger::init();

    let opts: Arguments = arguments().run();

    // Get an output stream handle to the default physical sound device
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let file_paths = search::get_file_paths(Path::new(&(opts.audio_path)), opts.recursive);

    // Create a new sink, which allows us to control the audio being played.
    let sink = Sink::try_new(&stream_handle).unwrap();

    sink.set_speed(opts.audio_speed);

    // Clone the sink so it can be shared across threads
    let sink: Arc<Mutex<Sink>> = Arc::new(Mutex::new(sink));

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

    interface::create_interface_thread(&sink);

    // Keep the main thread alive while audio is playing
    loop {
        thread::sleep(Duration::from_millis(100));
        let sink = sink.lock().unwrap();
        if sink.empty() {
            break;
        }
    }
}
