use std::{env, fs};
use std::io::{Read, stdin, stdout, Write};
use std::process::ExitStatus;
use std::time::Duration;

use clap::{arg, Arg, ArgAction, ArgMatches, Command};
use env_logger;
use log::{debug, trace};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use serde_json::{from_str, to_string};

// use serde_json::Value::String;
use models::*;

mod models;

fn main() {
    // set log level from args
    let matcher = Command::new("cli")
        .about("todo")
        .version("1.0")
        .author("Shubham")
        .arg(
            Arg::new("log")
                .short('l')
                .value_name("LEVEL")
                .required(false)
                .action(ArgAction::Append)
                .default_value("debug")
                .default_missing_value("debug")
                .num_args(1),
        )
        .arg(
            arg!([input] "users query")
                .trailing_var_arg(true)
                .num_args(1..),
        )
        .get_matches();

    set_log_level(&matcher);

    let user_query = matcher
        .try_get_many::<String>("input")
        .unwrap_or_else(|err| panic!("{}", err))
        .unwrap_or_else(|| panic!("no user input"))
        .cloned()
        .collect::<Vec<String>>()
        .join(" ");

    let context = init_and_get_context("/Users/shubham/.temp_his".to_string());
    let system_prompt = Prompts::get_system_prompt_2(&context);

    trace!("user query is {}", user_query);
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

    // let response_text = Client::new()
    //     .post("http://localhost:11434/api/chat")
    //     .json(&request_body)
    //     .timeout(Duration::from_secs(360))
    //     .send()
    //     .unwrap_or_else(|e| panic!("{}", e))
    //     .text() // we can directly use json() as well, and specify the type after the let
    //     // the issue is that the error messages are not clear in that case, thats why it is a 2 step process
    //     .unwrap_or_else(|e| panic!("{}", e));
    let response_text = DummyResponse::get_dummy_response();
    trace!("raw response is {}", response_text);

    let response = from_str::<OllamaResponse>(&response_text).unwrap_or_else(|e| panic!("{}", e));
    debug!(
        "response is {}",
        to_string(&response).unwrap_or("unable to deserialize response".to_string())
    );

    let suggestions: Vec<ModelSuggestion> =
        from_str::<OllamaPlaceholderResponse>(&response.message.content)
            .unwrap_or_else(|e| panic!("{}", e))
            .response;
    debug!(
        "suggestions are \n {}",
        to_string(&suggestions).unwrap_or("unable to deserialize suggestions".to_string())
    );

    // let content = r#"
    //  {"model":"qwen2.5","created_at":"2024-11-04T02:50:52.832969Z","message":{"role":"assistant","content":"{\n    \"response\": [\n        {\n            \"reasoning\": \"Based on the user's history, it seems they might be working on a Rust project and need to build or run it.\",\n            \"commands\": [\n                {\n                    \"cmd\": \"cargo build\",\n                    \"missing_fields\": [],\n                    \"reasoning\": \"This command builds the project. Since no missing fields are present, we can directly suggest this.\"\n                },\n                {\n                    \"cmd\": \"cargo run\",\n                    \"missing_fields\": [],\n                    \"reasoning\": \"After building, running the project is a common next step. No missing fields needed here.\"\n                }\n            ]\n        },\n        {\n            \"reasoning\": \"The user might want to open their `Cargo.toml` file in an editor since they are working on a Rust project.\",\n            \"commands\": [\n                {\n                    \"cmd\": \"nvim Cargo.toml\",\n                    \"missing_fields\": [],\n                    \"reasoning\": \"Opening the `Cargo.toml` file for potential modifications is likely.\"\n                }\n            ]\n        },\n        {\n            \"reasoning\": \"Given the presence of a `.git` directory, it's possible that the user wants to manage their project using Git.\",\n            \"commands\": [\n                {\n                    \"cmd\": \"git status\",\n                    \"missing_fields\": [],\n                    \"reasoning\": \"Checking the current state of the repository is a common first step after working on code.\"\n                },\n                {\n                    \"cmd\": \"git add .; git commit -m 'Adding changes to src directory'; git push\",\n                    \"missing_fields\": [\n                        {\"field\": \"commit_message\", \"suggested_value\": \"Adding changes to src directory\"}\n                    ],\n                    \"reasoning\": \"After making changes, committing and pushing them are typical next steps.\"\n                }\n            ]\n        },\n        {\n            \"reasoning\": \"The user might want to test their project locally or on another machine.\",\n            \"commands\": [\n                {\n                    \"cmd\": \"cargo test\",\n                    \"missing_fields\": [],\n                    \"reasoning\": \"Running tests is a common practice after making changes.\"\n                }\n            ]\n        },\n        {\n            \"reasoning\": \"The user might be interested in exploring the `src` directory to understand its contents.\",\n            \"commands\": [\n                {\n                    \"cmd\": \"tree src\",\n                    \"missing_fields\": [],\n                    \"reasoning\": \"Listing the contents of the `src` directory can help explore the project structure.\"\n                }\n            ]\n        },\n        {\n            \"reasoning\": \"Given the presence of a `.gitignore` file, it's possible that the user wants to ensure their files are tracked by Git.\",\n            \"commands\": [\n                {\n                    \"cmd\": \"cat .gitignore\",\n                    \"missing_fields\": [],\n                    \"reasoning\": \"Reviewing the contents of the `.gitignore` file can help manage version control.\"\n                }\n            ]\n        }\n    ]\n}"},"done_reason":"stop","total_duration":130625288958,"load_duration":28920875,"prompt_eval_count":1834,"prompt_eval_duration":20631456000,"eval_count":602,"eval_duration":109943901000}
    // "#;

    // check out iter mut
    // println!("enter you choice");
    // for (i, suggestion) in suggestions.iter().enumerate() {
    //     println!("{} - {} \n{}", i, suggestion.cmd, suggestion.reasoning);
    // }
    //
    // let mut user_choice_str = String::new();
    // _ = stdout().flush().expect("failed to flush stdout");
    // _ = stdin()
    //     .read_line(&mut user_choice_str)
    //     .expect("error in getting user input");
    //
    // debug!("user entered {}", user_choice_str.trim());
    //
    // // bubble up err to user
    // let user_choice_idx_res = validate_and_get_user_input_as_int(&suggestions, user_choice_str);
    // if user_choice_idx_res.is_err() {
    //     panic!("{}", user_choice_idx_res.unwrap_err());
    // }
    //
    // let user_choice = suggestions.get(user_choice_idx_res.unwrap()).unwrap();
    // debug!("user selected the command {}", user_choice.cmd);
    //
    // let mut execute_str = user_choice.cmd.clone();
    // if !user_choice.missing_fields.is_empty() {
    //     execute_str = get_missing_params_from_user(execute_str, user_choice.missing_fields.clone());
    // }
    //
    // println!("cmd to exec is {}", execute_str);
    //
    // execute_str.split("&&").for_each(|mut cmd| {
    //     cmd = cmd.trim();
    //     println!("executing cmd {}", cmd);
    //     let split_cmd = shell_words::split(&cmd).unwrap_or_else(|e| panic!("{}", e));
    //     let (name, args) = split_cmd
    //         .split_first()
    //         .unwrap_or_else(|| panic!("error in getting command"));
    //
    //     let mut command = std::process::Command::new(name);
    //     for arg in args {
    //         command.arg(arg);
    //     }
    //
    //     command.status().expect("command failed");
    // });
}

fn get_missing_params_from_user(mut cmd: String, missing_fields: Vec<MissingField>) -> String {
    debug!("start getting user input for command");
    for field in missing_fields {
        let mut value = String::new();
        _ = stdout().flush().expect("failed to flush stdout");
        println!("Enter the value for {} -> ", field.key);
        _ = stdin()
            .read_line(&mut value)
            .expect("error in getting user input");

        let pattern = format!("<{}>", field.key);
        cmd = cmd.replace(pattern.as_str(), value.trim());
    }

    return cmd;
}

fn validate_and_get_user_input_as_int(
    suggestions: &Vec<ModelSuggestion>,
    user_choice: String,
) -> Result<usize, CustomParserError> {
    let parsed_val_res = user_choice.trim().parse::<u32>();

    if parsed_val_res.is_err() {
        return Err(CustomParserError::ParseIntError(
            parsed_val_res.unwrap_err(),
        ));
    }

    let parsed_val = parsed_val_res.unwrap();

    let suggestion = suggestions.get(parsed_val as usize);
    if let None = suggestion {
        return Err(CustomParserError::OutOfBoundError(format!(
            "{} is out of bounds",
            parsed_val
        )));
    }

    Ok(parsed_val as usize)
}

fn set_log_level(matcher: &ArgMatches) {
    let log_level: String = match matcher.try_get_one::<String>("log") {
        Ok(s) => {
            if let Some(level) = s {
                level.to_string()
            } else {
                println!("log level is an optional of None, setting it to a default of debug");
                "debug".to_string()
            }
        }
        Err(e) => {
            println!("error while getting log level from user {}", e);
            println!("setting log level to a default of debug");
            "debug".to_string()
        }
    };

    match log_level.to_lowercase().as_str() {
        "error" | "e" => env::set_var("RUST_LOG", "error"),
        "warn" | "w" => env::set_var("RUST_LOG", "warn"),
        "info" | "i" => env::set_var("RUST_LOG", "warn"),
        "debug" | "d" => env::set_var("RUST_LOG", "debug"),
        "trace" | "t" => env::set_var("RUST_LOG", "trace"),
        _ => env::set_var("RUST_LOG", "debug"),
    }

    env_logger::init();
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
