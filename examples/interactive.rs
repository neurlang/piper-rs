use piper_rs::synth::PiperSpeechSynthesizer;
use rodio::buffer::SamplesBuffer;
use std::io::{self, BufRead};
use std::path::Path;

use std::fs::OpenOptions;
use std::io::{Write};

fn main() {
    // args: <config_path> [speaker_id] [sink device]
    let config_path = std::env::args().nth(1).expect("Please specify config path");
    let sid = std::env::args().nth(2);
    let output_path = std::env::args().nth(3).unwrap_or("".to_string()).to_string();

    // Load model/config (same as your original)
    let model = piper_rs::from_config_path(Path::new(&config_path)).unwrap();

    // Optional: set speaker id if provided
    if let Some(sid) = sid {
        let sid = sid.parse::<i64>().expect("Speaker ID should be number!");
        model.set_speaker(sid);
    }

    // Create synthesizer once
    let synth = PiperSpeechSynthesizer::new(model).unwrap();

    // Create audio output stream/handle once
    let (_stream, handle) = rodio::OutputStream::try_default().expect("Failed to open audio output");
    eprintln!("Ready. Type lines (one utterance per line). Ctrl-D (Unix) / Ctrl-Z (Windows) to finish.");

    // Read stdin line-by-line; each non-empty line is synthesized and played
    let stdin = io::stdin();
    let empty = output_path.is_empty();
    for line_res in stdin.lock().lines() {
        match line_res {
            Ok(line) => {
                let text = line.trim().to_string();
                if text.is_empty() {
                    continue;
                }

		println!("Synthesizing...");

                // Synthesize the line (same call you had)
                let audio = match synth.synthesize_parallel(text, None) {
                    Ok(a) => a,
                    Err(e) => {
                        eprintln!("Synthesis error: {e}");
                        continue;
                    }
                };

                // Collect samples into a single Vec<f32>
                let mut samples: Vec<f32> = Vec::new();
                for result in audio {
                    match result {
                        Ok(chunk) => samples.append(&mut chunk.into_vec()),
                        Err(e) => {
                            eprintln!("Chunk error: {e}");
                        }
                    }
                }


                // if there is pipe file path
                if !empty {

		  // Open FIFO for writing
                  let mut pipe = match OpenOptions::new()
		        .write(true)
		        .open(&output_path) {
                    Ok(a) => a,
                    Err(e) => {
                        eprintln!("Piping error: {e}");
                        continue;
                    }
                  };

		  // Convert f32 -> i16 and write
		  for &s in &samples {
		    let s16 = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                    match pipe.write_all(&s16.to_le_bytes()) {
                        Ok(a) => a,
                        Err(e) => {
                            eprintln!("Funneling error: {e}");
                            break;
                        }
                    };
		  }
                }


                // Create a sink, append buffer and block until finished
                // NOTE: sample rate is the same hard-coded value you used before (22050).
                // If your model uses a different sample rate, change this to match.
                let sink = rodio::Sink::try_new(&handle).expect("Failed to create sink");
                let buf = SamplesBuffer::new(1, 22050, samples);
                sink.append(buf);
		println!("Speaking...");
                sink.sleep_until_end();
		println!("Ready again.");

            }
            Err(e) => {
                eprintln!("Error reading stdin: {e}");
                break;
            }
        }
    }

    eprintln!("Done.");
}

