use std::{env, fs};
use std::io::Read;
use std::time::Duration;

use clap::{arg, Command};
use env_logger;
use log::{debug, trace};
use reqwest::blocking::Client;

use models::*;

mod models;

fn main() {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    // set log level from args
    let matcher = Command::new("cli")
        .about("todo")
        .version("1.0")
        .author("Shubham")
        .arg(
            arg!([input] "users query")
                .trailing_var_arg(true)
                .num_args(1..),
        )
        .get_matches();

    let user_query = matcher
        .try_get_many::<String>("input")
        .unwrap_or_else(|err| panic!("{}", err))
        .unwrap_or_else(|| panic!("no user input"))
        .cloned()
        .collect::<Vec<String>>()
        .join(" ");

    let context = init_and_get_context("/Users/shubham/.temp_his".to_string());
    let system_prompt = Prompts::get_system_prompt(&context);

    debug!("user query is {}", user_query);
    trace!(
        "context is {}",
        serde_json::to_string(&context).unwrap_or("unable to deserialize context".to_string())
    );
    trace!("system prompt is {}", system_prompt);

    let request_body = OllamaRequest {
        model: "qwen2.5".to_string(),
        format: "json".to_string(),
        stream: false,
        messages: vec![
            OllamaMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            OllamaMessage {
                role: "user".to_string(),
                content: user_query,
            },
        ],
    };

    let response_text = Client::new()
        .post("http://localhost:11434/api/chat")
        .json(&request_body)
        .timeout(Duration::from_secs(360))
        .send()
        .unwrap_or_else(|e| panic!("{}", e))
        .text() // we can directly use json() as well, and specify the type after the let
        // the issue is that the error messages are not clear in that case, thats why it is a 2 step process
        .unwrap_or_else(|e| panic!("{}", e));

    trace!("raw response is {}", response_text);

    let response: OllamaResponse =
        serde_json::from_str(&response_text).unwrap_or_else(|e| panic!("{}", e));

    debug!(
        "response is {}",
        serde_json::to_string(&response).unwrap_or("unable to deserialize response".to_string())
    );

    let suggestions: Vec<ModelSuggestion> =
        serde_json::from_str::<OllamaPlaceholderResponse>(&response.message.content)
            .unwrap_or_else(|e| panic!("{}", e))
            .response;

    debug!(
        "suggestions are \n {}",
        serde_json::to_string(&suggestions)
            .unwrap_or("unable to deserialize suggestions".to_string())
    );

    // check out iter mut
    for (i, suggestion) in suggestions.iter().enumerate() {
        println!("{} {} \n {}", i, suggestion.cmd, suggestion.reasoning);
    }

    // suggestions.iter().for_each(|s| match &s.missing_fields {
    //     None => debug!("no missing field"),
    //     Some(ss) => {
    //         if ss.len() == 0 {
    //             debug!("missing field is Some, but len is 0");
    //             return;
    //         }
    //
    //         println!("{}", ss.)
    //     }
    // })
}

fn init_and_get_context(his_file_path: String) -> Context {
    let history_file_path = match his_file_path.is_empty() {
        true => "/Users/shubham/.zli_history".to_string(),
        false => his_file_path,
    };

    debug!("history file path is {}", history_file_path);

    let mut buf = String::new();
    fs::File::open(history_file_path)
        .unwrap_or_else(|e| panic!("{}", e))
        .read_to_string(&mut buf)
        .unwrap_or_else(|e| panic!("{}", e));

    let history: Vec<History> = serde_json::from_str(&buf).unwrap_or_else(|e| panic!("{}", e));

    let cwd_path_buf = env::current_dir().unwrap_or_else(|e| panic!("{}", e));
    let cwd = cwd_path_buf
        .to_str()
        .unwrap_or_else(|| panic!("could not get cwd"))
        .to_string();

    let ls = cwd_path_buf
        .read_dir()
        .unwrap_or_else(|e| panic!("{}", e))
        .map(|file| {
            let f = file.unwrap_or_else(|e| panic!("{}", e));

            let is_file = f.file_type().unwrap_or_else(|e| panic!("{}", e)).is_file();
            let f_type = match is_file {
                true => "file".to_string(),
                false => "directory".to_string(),
            };

            File {
                name: f.file_name().to_str().unwrap().to_string(),
                kind: f_type.to_string(),
            }
        })
        .collect();

    return Context { cwd, ls, history };
}
