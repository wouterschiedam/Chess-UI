use std::error::Error;
use std::path::PathBuf;
use std::time::Duration;

use iced::futures::channel::mpsc::Sender;
use iced::{subscription, Subscription};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, ChildStdout};
use tokio::sync::mpsc::Receiver;
use tokio::time::{self};

use super::config::Clock;
use super::engine_processing::{handle_engine_thinking, start_engine};
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
    pub clock: Clock,
}

impl UIengine {
    pub fn new(path: String, depth: u32) -> Self {
        Self {
            engine_path: PathBuf::from(path),
            search_up_to: depth,
            //position: String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
            position: String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"),
            clock: Clock::new(),
        }
    }

    pub fn run_engine(self) -> Subscription<Message> {
        subscription::channel(
            std::any::TypeId::of::<UIengine>(),
            100,
            move |mut output| {
                let engine1 = self.clone();

                async move {
                    let mut state1 = EngineState::Start(engine1.clone());

                    loop {
                        state1 = match run_single_engine(state1, &engine1, &mut output).await {
                            Ok(new_state) => new_state,
                            Err(e) => {
                                eprintln!("Engine 1 encountered an error: {}", e);
                                EngineState::TurnedOff
                            }
                        };
                    }
                }
            },
        )
    }
}

async fn run_single_engine(
    mut state: EngineState,
    engine: &UIengine,
    output: &mut Sender<Message>,
) -> Result<EngineState, Box<dyn Error>> {
    match &mut state {
        EngineState::Start(engine) => {
            match start_engine(&engine.engine_path, &engine.position, engine.search_up_to).await {
                Ok((process, receiver, sender)) => {
                    output.try_send(Message::EngineReady(sender))?;
                    Ok(EngineState::Thinking(
                        process,
                        engine.search_up_to,
                        receiver,
                    ))
                }
                Err(e) => {
                    eprintln!("Failed to start engine: {}", e);
                    Ok(EngineState::TurnedOff)
                }
            }
        }
        EngineState::Thinking(process, search_up_to, receiver) => {
            handle_engine_thinking(process, *search_up_to, receiver, output, &engine.clock).await?;
            Ok(state)
        }
        EngineState::TurnedOff => {
            tokio::time::sleep(Duration::from_millis(10)).await;
            Ok(EngineState::TurnedOff)
        }
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
