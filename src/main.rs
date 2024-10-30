use std::{env, fs};
use std::io::{Read, stdin, stdout, Write};
use std::time::Duration;

use clap::{arg, Arg, ArgAction, ArgMatches, Command};
use env_logger;
use log::{debug, trace};
use reqwest::blocking::Client;

// use serde_json::Value::String;
use models::*;

mod models;

fn main() {
    // set log level from args
    // let matcher = Command::new("cli")
    //     .about("todo")
    //     .version("1.0")
    //     .author("Shubham")
    //     .arg(
    //         Arg::new("log")
    //             .short('l')
    //             .value_name("LEVEL")
    //             .required(false)
    //             .action(ArgAction::Append)
    //             .default_value("debug")
    //             .default_missing_value("debug")
    //             .num_args(1),
    //     )
    //     .arg(
    //         arg!([input] "users query")
    //             .trailing_var_arg(true)
    //             .num_args(1..),
    //     )
    //     .get_matches();
    //
    // set_log_level(&matcher);
    //
    // let user_query = matcher
    //     .try_get_many::<String>("input")
    //     .unwrap_or_else(|err| panic!("{}", err))
    //     .unwrap_or_else(|| panic!("no user input"))
    //     .cloned()
    //     .collect::<Vec<String>>()
    //     .join(" ");
    //
    // let context = init_and_get_context("/Users/shubham/.temp_his".to_string());
    // let system_prompt = Prompts::get_system_prompt(&context);
    //
    // debug!("user query is {}", user_query);
    // trace!(
    //     "context is {}",
    //     serde_json::to_string(&context).unwrap_or("unable to deserialize context".to_string())
    // );
    // trace!("system prompt is {}", system_prompt);
    //
    // let request_body = OllamaRequest {
    //     model: "qwen2.5".to_string(),
    //     format: "json".to_string(),
    //     stream: false,
    //     messages: vec![
    //         OllamaMessage {
    //             role: "system".to_string(),
    //             content: system_prompt,
    //         },
    //         OllamaMessage {
    //             role: "user".to_string(),
    //             content: user_query,
    //         },
    //     ],
    // };
    //
    // let response_text = Client::new()
    //     .post("http://localhost:11434/api/chat")
    //     .json(&request_body)
    //     .timeout(Duration::from_secs(360))
    //     .send()
    //     .unwrap_or_else(|e| panic!("{}", e))
    //     .text() // we can directly use json() as well, and specify the type after the let
    //     // the issue is that the error messages are not clear in that case, thats why it is a 2 step process
    //     .unwrap_or_else(|e| panic!("{}", e));
    //
    // trace!("raw response is {}", response_text);
    //
    // let response: OllamaResponse =
    //     serde_json::from_str(&response_text).unwrap_or_else(|e| panic!("{}", e));
    //
    // debug!(
    //     "response is {}",
    //     serde_json::to_string(&response).unwrap_or("unable to deserialize response".to_string())
    // );
    //
    // let suggestions: Vec<ModelSuggestion> =
    //     serde_json::from_str::<OllamaPlaceholderResponse>(&response.message.content)
    //         .unwrap_or_else(|e| panic!("{}", e))
    //         .response;
    //
    // debug!(
    //     "suggestions are \n {}",
    //     serde_json::to_string(&suggestions)
    //         .unwrap_or("unable to deserialize suggestions".to_string())
    // );
    //
    // // check out iter mut
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
    // let mut cmd_to_exec = user_choice.cmd.clone();
    // if !user_choice.missing_fields.is_empty() {
    //     cmd_to_exec = mass_replace_in_string(cmd_to_exec, user_choice.missing_fields.clone());
    // }
    //
    // println!("cmd to exec is {}", cmd_to_exec);

    // let split_cmd = shell_words::split(&cmd_to_exec).unwrap_or_else(|e| panic!("{}", e));
    // let (cmd, args) = split_cmd
    //     .split_first()
    //     .unwrap_or_else(|| panic!("error in getting command"));

    let status = std::process::Command::new("git")
        .arg("add")
        .arg("-A")
        .spawn()
        .expect("git command failed to start");
}

fn mass_replace_in_string(mut cmd: String, missing_fields: Vec<MissingField>) -> String {
    for field in missing_fields {
        let mut value = String::new();
        _ = stdout().flush().expect("failed to flush stdout");
        print!("Enter the value for {} -> ", field.key);
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
        "error" => env::set_var("RUST_LOG", "error"),
        "warn" => env::set_var("RUST_LOG", "warn"),
        "info" => env::set_var("RUST_LOG", "warn"),
        "debug" => env::set_var("RUST_LOG", "debug"),
        "trace" => env::set_var("RUST_LOG", "trace"),
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
