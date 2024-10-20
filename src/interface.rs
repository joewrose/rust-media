use std::{
    io::Write,
    sync::{Arc, Mutex},
};

use rodio::Sink;

// String constants
const HELP_MESSAGE: &str = r#"
Available commands:
  pause       - Pause audio playback.
  resume      - Resume audio playback.
  skip        - Skip to the next track
  speed <n>   - Set playback speed to a specific value (min: 0.1, max: 2.0)
  quit        - Stop playback and exit the program.
  help        - Show this help message.
"#;

fn set_speed(sink: &rodio::Sink, speed: f32) {
    sink.set_speed(speed.max(0.1).min(2.0));
}

pub(crate) fn create_interface_thread(sink: &Arc<Mutex<Sink>>) {
    // Spawn a thread to handle user input
    let sink_clone = sink.clone();
    std::thread::spawn(move || {
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
}
