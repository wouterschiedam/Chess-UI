use std::error::Error;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use iced::futures::channel::mpsc::Sender;
use tokio::io::{AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::time::timeout;

use crate::extra::parse::algebraic_square_to_number;

use super::config::Clock;
use super::engine::{read_moves_from_process, read_setup_from_process};
use super::ui::Message;
pub async fn start_engine(
    engine_path: &PathBuf,
    position: &str,
    search_up_to: u32,
) -> Result<
    (
        Child,
        tokio::sync::mpsc::Receiver<String>,
        tokio::sync::mpsc::Sender<String>,
    ),
    Box<dyn Error>,
> {
    let (sender, receiver): (
        tokio::sync::mpsc::Sender<String>,
        tokio::sync::mpsc::Receiver<String>,
    ) = tokio::sync::mpsc::channel(100);

    let mut cmd = Command::new(engine_path);
    cmd.kill_on_drop(true)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped());

    let mut process = cmd.spawn().expect("Error starting engine");

    let pos = format!("position fen {}\n", position);
    if let Some(stdin) = process.stdin.as_mut() {
        stdin.write_all(b"uci\n").await?;
        stdin.flush().await?;
    }

    let reader = BufReader::new(process.stdout.as_mut().expect("Failed to get stdout"));
    let mut buffer_str = String::new();
    if !read_setup_from_process(reader, &mut buffer_str).await {
        return Err("Failed to initialize engine".into());
    }

    if let Some(stdin) = process.stdin.as_mut() {
        stdin.write_all(b"isready\n").await?;
        stdin.flush().await?;
    }

    let reader = BufReader::new(process.stdout.as_mut().expect("Failed to get stdout"));
    if !read_setup_from_process(reader, &mut buffer_str).await {
        return Err("Engine not ready".into());
    }

    if let Some(stdin) = process.stdin.as_mut() {
        stdin.write_all(pos.as_bytes()).await?;
        stdin.flush().await?;
    }

    Ok((process, receiver, sender))
}

pub async fn handle_engine_thinking(
    process: &mut Child,
    search_up_to: u32,
    receiver: &mut tokio::sync::mpsc::Receiver<String>,
    output: &mut Sender<Message>,
    clock: &Clock,
) -> Result<(), Box<dyn Error>> {
    if let Some(message) = receiver.recv().await {
        if message == "stop" || message == "quit" {
            if let Some(stdin) = process.stdin.as_mut() {
                stdin.write_all(b"quit\n").await?;
                stdin.flush().await?;
            }

            let terminate_timeout = timeout(Duration::from_millis(1000), process.wait()).await;
            if terminate_timeout.is_err() {
                let _ = timeout(Duration::from_millis(500), process.kill()).await;
            }

            output.try_send(Message::EngineStopped(true))?;
            return Ok(());
        }

        let pos = format!("position fen {}\n", message);
        let limit = format!(
            "go depth {} wtime {} btime {}\n",
            search_up_to, clock.wtime, clock.btime
        );

        if let Some(stdin) = process.stdin.as_mut() {
            stdin.write_all(pos.as_bytes()).await?;
            stdin.flush().await?;
            stdin.write_all(limit.as_bytes()).await?;
            stdin.flush().await?;
        }

        let reader = BufReader::new(process.stdout.as_mut().expect("Failed to get stdout"));
        let mut buffer_str = String::new();
        let response = read_moves_from_process(reader, &mut buffer_str).await?;

        let bestmove = response.last().unwrap().split_whitespace().nth(1).unwrap();
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

    Ok(())
}
