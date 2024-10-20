use rodio::Sink;
use rodio::{Decoder, OutputStream};
use std::fs::File;
use std::io::BufReader;

fn main() {
    // Get an output stream handle to the default physical sound device
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    // Load a sound from a file, using a path relative to Cargo.toml
    let file =
        BufReader::new(File::open("/home/joewrose/Projects/Rust Media Player/test.mp3").unwrap());

    // Decode that sound file into a source
    let source = Decoder::new(file).unwrap();

    // Create a new sink, which allows us to control the audio being played.
    let sink = Sink::try_new(&stream_handle).unwrap();

    sink.append(source);

    // The audio is played in a separate thread, this call makes the current thread sleep until it is done
    sink.sleep_until_end();
}
