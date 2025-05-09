use chatgpt::prelude::*;
use chatgpt::types::Role;
use futures_util::stream::StreamExt;
use std::io::{stdout, Write};
use std::fs;
use crate::commands::{Command, Input, print_msg, clear_console};

pub async fn stream_single_response(client: &ChatGPT, message: String, prompt_path: String) -> Result<()> {
    let system_prompt = load_system_prompt(prompt_path);
    let history: Vec<ChatMessage> = vec![
        ChatMessage {
            role: Role::System,
            content: system_prompt,
        },
        ChatMessage {
            role: Role::User,
            content: message,
        },
    ];
    // let width = term_size::dimensions().unwrap_or((80, 24)).0;
    // let separator = "-".repeat(width);
    // println!("{}", separator);
    // let mut line_index: usize = 0;
    let mut stream = client.send_history_streaming(&history).await?;
    // stream.for_each(|each| async move {
    //     match each {
    //         ResponseChunk::Content { delta, response_index: _ } => {
    //             print!("{}", delta);
    //             stdout().lock().flush().unwrap();
    //         }
    //         _ => {}
    //     }
    // }).await;
    println!();
    while let Some(chunk) = stream.next().await {
        match chunk {
            ResponseChunk::Content { delta, response_index: _ } => {
                print!("{}", delta);
                stdout().lock().flush().unwrap();
            }
            _ => {}
        }
    }

    println!("\n");
    std::thread::sleep(std::time::Duration::from_millis(500));
    Ok(())
}

pub async fn start_conversation(client: &ChatGPT, prompt_path: String) -> Result<()> {
    // let mut conversation: Conversation = client.new_conversation();
    let system_prompt = load_system_prompt(prompt_path);
    let mut conversation: Conversation = client.new_conversation_directed(system_prompt.clone());
    loop {
        let input = get_input();
        println!();
        match input {
            Input::Message(message) => {
                let output = stream_next_response(&mut conversation, message).await?;
                append_response(&mut conversation, output);
            }
            Input::Command(command) => {
                match command {
                    Command::Exit => {
                        println!("Exiting...");
                        return Ok(());
                    }
                    Command::Clear => {
                        // conversation.history.clear();
                        conversation.history = vec![
                            ChatMessage {
                                role: Role::System,
                                content: system_prompt.clone(),
                            },
                        ];
                        clear_console();
                        continue;
                    }
                    Command::History => {
                        for msg in &conversation.history {
                            print_msg(msg);
                        }
                        continue;
                    }
                    Command::PrintPrompt => {
                        println!("System prompt:\n{}", system_prompt);
                        continue;
                    }
                    Command::Help => {
                        println!("Type your message and press Enter to send it.");
                        println!("Type 'exit' or '/q' to quit the program.");
                        println!("Type 'clear' or '/c' to clear the conversation history.");
                        println!("Type 'history' or '/h' to view the conversation history.");
                        println!("Type 'system' to view the system prompt.");
                        println!("Type 'help' or '?' for this help message.");
                        continue;
                    }
                }
            }
            Input::Invalid => {
                println!("Please enter a valid message.");
                continue;
            }
        };
        
    }
}

fn load_system_prompt(prompt_path: String) -> String {
    fs::read_to_string(prompt_path).unwrap_or_else(|_| {
        eprintln!("Failed to read system prompt from file. Please ensure the file exists.");
        std::process::exit(1);
    })
}

fn get_input() -> Input {
    let mut input = String::new();
    print!("> ");
    stdout().lock().flush().unwrap();   
    std::io::stdin().read_line(&mut input).expect("Failed to read line");
    let input = input.trim();
    if input.is_empty() {
        return Input::Invalid;
    }
    match input {
        "exit" | "quit" | "/q" | "/x" => Input::Command(Command::Exit),
        "clear" | "/c" => Input::Command(Command::Clear),
        "history" | "/h" => Input::Command(Command::History),
        "prompt" | "/p"=> Input::Command(Command::PrintPrompt),
        "help" | "?" | "/" => Input::Command(Command::Help),
        _ => Input::Message(input.to_string()),
    }
}

async fn stream_next_response(conversation: &mut Conversation, message: String) -> Result<Vec<ResponseChunk>> {
    if message.is_empty() {
        println!("Please enter a valid message.");
        return Ok(vec![]);
    }
    let mut stream = conversation.send_message_streaming(message).await?;
    let mut output: Vec<ResponseChunk> = Vec::new();
    while let Some(chunk) = stream.next().await {
        match chunk {
            ResponseChunk::Content {
                delta,
                response_index,
            } => {
                print!("{}", delta);
                stdout().lock().flush().unwrap();
                output.push(ResponseChunk::Content {
                    delta,
                    response_index,
                });
            }
            other => output.push(other),
        }
    }
    std::thread::sleep(std::time::Duration::from_millis(500));
    println!("\n");
    Ok(output)
}

fn append_response(conversation: &mut Conversation, output: Vec<ResponseChunk>) {
    let messages = ChatMessage::from_response_chunks(output);
    conversation.history.push(messages[0].to_owned());
    // dbg!(&conversation.history); // Debugging to see the last message in the conversation history
}
