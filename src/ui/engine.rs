use std::error::Error;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use iced::keyboard::KeyCode;
use iced::{subscription, Subscription};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdout, Command};
use tokio::sync::mpsc::Receiver;
use tokio::time::{self, timeout};

use crate::extra::parse::algebraic_square_to_number;

use super::ui::Message;

#[derive(Debug, PartialEq)]
pub enum EngineStatus {
    TurnedOff,
    TurnedOn,
}

pub enum EngineState {
    Start(UIengine),
    Thinking(Child, u32, Receiver<String>),
    TurnedOff,
}

#[derive(Debug, Clone)]
pub struct UIengine {
    pub engine_path: PathBuf,
    pub search_up_to: u32,
    pub position: String,
}

impl UIengine {
    pub fn new() -> Self {
        Self {
            engine_path: PathBuf::from(
                "/Users/wouter/personal/rust/chess-engine/target/release/chess-engine",
            ),
            search_up_to: 3,
            //position: String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
            position: String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
        }
    }

    pub fn run_engine(engine: UIengine) -> Subscription<Message> {
        subscription::channel(
            std::any::TypeId::of::<UIengine>(),
            100,
            move |mut output| {
                let engine = engine.clone();
                async move {
                    let mut state = EngineState::Start(engine);

                    loop {
                        match &mut state {
                            EngineState::Start(engine) => {
                                // Create mspc channel with sender and receiver
                                let (sender, receiver): (
                                    tokio::sync::mpsc::Sender<String>,
                                    tokio::sync::mpsc::Receiver<String>,
                                ) = tokio::sync::mpsc::channel(100);

                                // Create new command to start the engine
                                let mut cmd = Command::new(&engine.engine_path);

                                cmd.kill_on_drop(true)
                                    .stdin(Stdio::piped())
                                    .stdout(Stdio::piped());

                                // Start chess engine
                                let mut process = cmd.spawn().expect("Error starting engine");

                                // Some local variables used to start communication with the engine.
                                let pos = String::from("position fen ")
                                    + &engine.position
                                    + &String::from("\n");

                                // Write to stdin asynchronously
                                if let Some(stdin) = process.stdin.as_mut() {
                                    // You can use write_all to write a slice of bytes asynchronously
                                    stdin
                                        .write_all(b"uci\n")
                                        .await
                                        .expect("Error writing to stdin");

                                    // Optionally, flush the buffer
                                    stdin.flush().await.expect("Error flushing stdin");
                                }

                                let reader = BufReader::new(
                                    process.stdout.as_mut().expect("Failed to get stdout"),
                                );

                                let mut buffer_str = String::new();

                                // read input from engine
                                let uciok = read_setup_from_process(reader, &mut buffer_str).await;

                                if uciok {
                                    if let Some(stdin) = process.stdin.as_mut() {
                                        // You can use write_all to write a slice of bytes asynchronously
                                        stdin
                                            .write_all(b"isready\n")
                                            .await
                                            .expect("Error writing to stdin");

                                        // Optionally, flush the buffer
                                        stdin.flush().await.expect("Error flushing stdin");
                                    }

                                    let reader = BufReader::new(
                                        process.stdout.as_mut().expect("Failed to get stdout"),
                                    );

                                    let readyok =
                                        read_setup_from_process(reader, &mut buffer_str).await;

                                    if readyok {
                                        if let Some(stdin) = process.stdin.as_mut() {
                                            stdin
                                                .write_all(
                                                    format!("position fen {} \n", pos).as_bytes(),
                                                )
                                                .await
                                                .expect("Error writing to stdin");

                                            stdin.flush().await.expect("Error flushing stdin");
                                        }

                                        output.try_send(Message::EngineReady(sender)).expect(
                                            "Error on the mpsc channel in the engine subscription",
                                        );

                                        state = EngineState::Thinking(
                                            process,
                                            engine.search_up_to,
                                            receiver,
                                        );

                                        continue;
                                    }
                                }

                                // Send quit command to engine
                                if let Some(stdin) = process.stdin.as_mut() {
                                    stdin
                                        .write_all(b"quit\n")
                                        .await
                                        .expect("Error stopping the engine");
                                }

                                eprintln!("Engine took too long to start, aborting...");
                                let terminate_timeout =
                                    timeout(Duration::from_millis(1000), process.wait()).await;
                                if let Err(_) = terminate_timeout {
                                    eprintln!("Engine didn't quit, killing the process now...");
                                    let kill_result =
                                        timeout(Duration::from_millis(500), process.kill()).await;
                                    if let Err(e) = kill_result {
                                        eprintln!("Error killing the engine process: {e}");
                                    }
                                    eprintln!("Engine stopped");
                                }

                                output.try_send(Message::EngineStopped(false)).expect(
                                    "Error in the mspc channel in the subscription channel",
                                );
                                state = EngineState::TurnedOff;
                            }
                            EngineState::Thinking(process, search_up_to, receiver) => {
                                let message = receiver.recv().await;

                                if let Some(message) = message {
                                    if &message == "stop" || &message == "quit" {
                                        // Send quit command to engine
                                        if let Some(stdin) = process.stdin.as_mut() {
                                            stdin
                                                .write_all(b"quit\n")
                                                .await
                                                .expect("Error stopping the engine");

                                            stdin.flush().await.expect("Error flushing stdin");
                                        }

                                        // kill process
                                        let terminate_timeout =
                                            timeout(Duration::from_millis(1000), process.wait())
                                                .await;
                                        if let Err(_) = terminate_timeout {
                                            eprintln!(
                                                "Engine didn't quit, killing the process now..."
                                            );
                                            let kill_result =
                                                timeout(Duration::from_millis(500), process.kill())
                                                    .await;
                                            if let Err(e) = kill_result {
                                                eprintln!("Error killing the engine process: {e}");
                                            }
                                        }

                                        output.try_send(Message::EngineStopped(true)).expect(
                                            "Error on the mspc channel in the engine subscription",
                                        );
                                        state = EngineState::TurnedOff;
                                        continue;
                                    } else {
                                        let pos = String::from("position fen ")
                                            + &message
                                            + &String::from("\n");

                                        let limit = String::from("go depth ")
                                            + &search_up_to.to_string()
                                            + &String::from("\n");

                                        println!("Thinking: {}depth: {}", pos, limit);

                                        if let Some(stdin) = process.stdin.as_mut() {
                                            stdin
                                                .write_all(pos.as_bytes())
                                                .await
                                                .expect("Error communicating with the engine");

                                            stdin.flush().await.expect("Error flushing stdin");

                                            stdin
                                                .write_all(limit.as_bytes())
                                                .await
                                                .expect("Error communicating with the engine");

                                            stdin.flush().await.expect("Error flushing stdin");
                                        }
                                    }
                                }

                                let mut eval: Option<KeyCode> = None;
                                let mut bestmove: Option<KeyCode> = None;

                                // Get results from search
                                let reader = BufReader::new(
                                    process.stdout.as_mut().expect("Failed to get stdout"),
                                );

                                let mut buffer_str = String::new();
                                let response =
                                    match read_moves_from_process(reader, &mut buffer_str).await {
                                        Ok(lines) => lines,
                                        Err(error) => {
                                            eprintln!("Error reading from stdin {}", error);
                                            vec![error.to_string()]
                                        }
                                    };

                                let bestmove =
                                    response[3].split_whitespace().collect::<Vec<&str>>()[1];

                                output
                                    .try_send(Message::SelectSquare(algebraic_square_to_number(
                                        &bestmove[0..2],
                                    )))
                                    .expect("Error on the mspc channel in the engine subscription");

                                output
                                    .try_send(Message::SelectSquare(algebraic_square_to_number(
                                        &bestmove[2..4],
                                    )))
                                    .expect("Error on the mspc channel in the engine subscription");
                            }
                            EngineState::TurnedOff => {
                                tokio::time::sleep(std::time::Duration::from_millis(10)).await
                            }
                        }
                    }
                }
            },
        )
    }
}

pub async fn read_setup_from_process(
    mut reader: BufReader<&mut ChildStdout>,
    mut buffer_str: &mut String,
) -> bool {
    loop {
        let read_line_result = async {
            reader
                .read_line(&mut buffer_str)
                .await
                .map(|_| buffer_str.clone())
        };

        match time::timeout(Duration::from_millis(3000), read_line_result).await {
            Ok(Ok(line)) => {
                if line.contains("uciok") || line.contains("readyok") {
                    buffer_str.clear();
                    return true;
                }
            }
            Ok(Err(e)) => {
                eprintln!("Error reading line: {:?}", e);
                break false;
            }
            Err(e) => {
                eprintln!("Timeout occurred: {}", e);
                break false;
            }
        }
    }
}

pub async fn read_moves_from_process(
    mut reader: BufReader<&mut ChildStdout>,
    mut buffer_str: &mut String,
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut lines = Vec::new();
    loop {
        let read_line_result = async {
            reader
                .read_line(&mut buffer_str)
                .await
                .map(|_| buffer_str.clone())
        };

        match time::timeout(Duration::from_millis(7000), read_line_result).await {
            Ok(Ok(line)) => {
                buffer_str.clear();
                if line.contains("bestmove") {
                    lines.push(line);
                    break;
                }
                lines.push(line);
            }
            Ok(Err(e)) => {
                eprintln!("Error reading line: {:?}", e);
                return Err(Box::new(e)); // Return an error with more information
            }
            Err(e) => {
                eprintln!("Timeout occurred: {}", e);
                return Err(Box::new(e));
            }
        }
    }

    Ok(lines)
}
