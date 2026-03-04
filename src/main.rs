use async_openai::{Client, config::OpenAIConfig};
use clap::Parser;
use serde_json::{Value, json};
use std::{env, fs, process};

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
    let mut messages: Vec<Value> = Vec::new();
    messages.push(json!({
            "role": "user",
            "content": args.prompt
    }));
    loop {
        #[allow(unused_variables)]
        let response: Value = client
            .chat()
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

        // You can use print statements as follows for debugging, they'll be visible when running tests.
        eprintln!("Logs from your program will appear here!");

        // TODO: Uncomment the lines below to pass the first stage
        let message = &response["choices"][0]["message"];

        if let Some(tools) = message["tool_calls"].as_array()
            && !tools.is_empty()
        {
            messages.push(message.to_owned());
            let tool_call = match &tools[0]["function"] {
                Value::Object(tool) => tool,
                _ => panic!("Invalid tool call"),
            };
            let tool_call_id = match &tools[0]["id"] {
                Value::String(id) => id,
                _ => panic!("Tool call id not provided or not a string"),
            };

            let tool_name = match &tool_call["name"] {
                Value::String(name) => name,
                _ => panic!("Tool name must be a string"),
            };

            let args: Value = match &tool_call["arguments"] {
                Value::String(raw) => serde_json::from_str(raw.as_str()).unwrap(),
                _ => panic!("Invalid arguments"),
            };

            match tool_name.as_str() {
                "Read" => {
                    if let Some(path) = args["file_path"].as_str() {
                        let file_content = fs::read_to_string(path)?;
                        messages.push(json!({
                            "role": "tool",
                            "tool_call_id": tool_call_id,
                            "content": file_content,
                        }));
                    } else {
                        panic!("file_path must be a string")
                    }
                }
                _ => panic!("file path must be a string"),
            };
        } else if let Some(content) = response["choices"][0]["message"]["content"].as_str() {
            println!("{}", content);
            break;
        }
    }

    Ok(())
}
