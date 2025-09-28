use piper_rs::synth::PiperSpeechSynthesizer;
use rodio::buffer::SamplesBuffer;
use std::io::{self, BufRead};
use std::path::Path;

fn main() {
    // args: <config_path> [speaker_id]
    let config_path = std::env::args().nth(1).expect("Please specify config path");
    let sid = std::env::args().nth(2);

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
    for line_res in stdin.lock().lines() {
        match line_res {
            Ok(line) => {
                let text = line.trim().to_string();
                if text.is_empty() {
                    continue;
                }

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

                // Create a sink, append buffer and block until finished
                // NOTE: sample rate is the same hard-coded value you used before (22050).
                // If your model uses a different sample rate, change this to match.
                let sink = rodio::Sink::try_new(&handle).expect("Failed to create sink");
                let buf = SamplesBuffer::new(1, 22050, samples);
                sink.append(buf);
                sink.sleep_until_end();
            }
            Err(e) => {
                eprintln!("Error reading stdin: {e}");
                break;
            }
        }
    }

    eprintln!("Done.");
}

