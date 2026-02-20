use async_openai::{Client, config::OpenAIConfig};
use clap::Parser;
use serde_json::{Value, json};
use std::{env, process};

mod read;

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'p', long)]
    prompt: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let base_url = env::var("OPENROUTER_BASE_URL")
        .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());

    let api_key = env::var("OPENROUTER_API_KEY").unwrap_or_else(|_| {
        eprintln!("OPENROUTER_API_KEY is not set");
        process::exit(1);
    });

    let config = OpenAIConfig::new()
        .with_api_base(base_url)
        .with_api_key(api_key);

    let client = Client::with_config(config);

    let mut messages = json!([
        {
            "role": "user",
            "content": args.prompt
        }
    ]);

    if let Some(content) = loop {
        let response: Value = client
            .chat()
            // seems to mean "bring your own type" and is littered everywhere in the crate
            .create_byot(json!({
                "messages": messages,
                "model": "anthropic/claude-haiku-4.5",
                "tools": [
                    {
                        "type": "function",
                        "function": {
                            "name": "Read",
                            "description": "Read and return the contents of a file",
                            "parameters": {
                            "type": "object",
                            "properties": {
                                "file_path": {
                                "type": "string",
                                "description": "The path to the file to read"
                                }
                            },
                            "required": ["file_path"]
                            }
                        }
                    }
                ]
            }))
            .await?;
        eprintln!(
            "Assistant returned {} choices.",
            response["choices"].as_array().unwrap().len()
        );
        let choice = &response["choices"][0];
        messages
            .as_array_mut()
            .unwrap()
            .push(choice["message"].clone());

        if choice["finish_reason"].as_str() == Some("stop")
            || choice["message"]["tool_calls"].as_array().is_none()
        {
            break choice["message"]["content"].as_str().map(|c| c.to_string());
        }

        eprintln!(
            "Assistant picked {} tools.",
            choice["message"]["tool_calls"].as_array().unwrap().len()
        );
        let call = &choice["message"]["tool_calls"][0];
        let id = call["id"].as_str().unwrap();
        if let Some(tool) = call["function"].as_object() {
            let name = tool["name"].as_str().unwrap();
            let args: read::Args =
                serde_json::from_str(tool["arguments"].as_str().unwrap()).unwrap();
            if let Ok(content) = read::read(args) {
                eprintln!("Tool call {} (ID {}) created output:", name, id);
                eprintln!("{}", content);
                messages.as_array_mut().unwrap().push(json!({
                    "role": "tool",
                    "tool_call_id": id,
                    "content": content
                }));
            } else {
                eprintln!("Tool call {} (ID {}) failed", name, id);
            }
        } else {
            eprintln!("Tool call ID {} function parse failed", id);
        }
    } {
        println!("{}", content);
    } else {
        eprintln!("Empty message content");
    }

    Ok(())
}
