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

    let response: Value = client
        .chat()
        // seems to mean "bring your own type" and is littered everywhere in the crate
        .create_byot(json!({
            "messages": [
                {
                    "role": "user",
                    "content": args.prompt
                }
            ],
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

    // You can use print statements as follows for debugging, they'll be visible when running tests.
    eprintln!("Logs from your program will appear here!");

    if let Some(calls) = response["choices"][0]["message"]["tool_calls"].as_array() {
        let call = &calls[0];
        if let Some(call) = call["function"].as_object() {
            let _name = call["name"].as_str().unwrap();
            let args = call["arguments"].as_str().unwrap();
            let args: read::Args = serde_json::from_str(args).unwrap();
            if let Ok(contents) = read::read(args) {
                println!("{}", contents);
            }
        }
    }
    if let Some(content) = response["choices"][0]["message"]["content"].as_str() {
        println!("{}", content);
    }

    Ok(())
}
