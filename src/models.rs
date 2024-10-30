use std::fmt::{Debug, Display, Formatter};
use std::num::ParseIntError;

use serde::{Deserialize, Serialize};
use serde_json::to_string;

// parse errors
pub enum CustomParserError {
    ParseIntError(ParseIntError),
    OutOfBoundError(String),
}

impl Debug for CustomParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CustomParserError::ParseIntError(e) => write!(f, "ParseIntError: {:?}", e),
            CustomParserError::OutOfBoundError(msg) => write!(f, "OutOfBoundError: {}", msg),
        }
    }
}

impl Display for CustomParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CustomParserError::ParseIntError(e) => write!(f, "Failed to parse integer: {}", e),
            CustomParserError::OutOfBoundError(msg) => write!(f, "Out of bounds error: {}", msg),
        }
    }
}

impl std::error::Error for CustomParserError {}

// context
#[derive(Serialize, Deserialize)]
pub struct Context {
    pub cwd: String,
    pub ls: Vec<File>,
    pub history: Vec<History>,
}

#[derive(Serialize, Deserialize)]
pub struct History {
    pub dir: String,
    pub cmd: String,
    pub datetime: String,
}

#[derive(Serialize, Deserialize)]
pub struct File {
    pub name: String,
    pub kind: String,
}

// prompts
pub struct Prompts;

impl Prompts {
    pub fn get_system_prompt(ctx: &Context) -> String {
        let ctx_string = to_string(ctx).unwrap_or_else(|e| panic!("{}", e));

        format!(
            r#"You are a command line terminal assistant. Your primary job is to help the user perform
            tasks in the terminal by suggesting up to 5 commands that might satisfy their needs.

            You will be provided a context from the user to help you help them. The context will be
            given to you as a JSON string with the following schema:

            context = {{
              "cwd": string // current working directory of the user,
              "files_in_cwd": []file_entry // list of files in the current working directory,
              "history": []history_entry // command line history of the commands executed by the user
            }}

            file_entry = {{
              "name": string // name of the file,
              "kind": string // what kind of a file is it, i.e., file, directory, etc.
            }}

            history_entry = {{
              "dir": string // directory where the command was run,
              "command": string // command which was run,
              "datetime": string // when the command was run
            }}

            The actual context will be given below.

            Your output should always be a valid JSON with the following schema:

            output = {{
              "response": [] suggestion // list of up to 5 suggestions
            }}

            suggestion = {{
              "cmd": string // command you think will help the user; if there is a missing field in
              the command it should be enclosed in <> brackets this is very important,
              "missing_fields": [] missing_fields // list of fields you might need to make the
              command a valid command; should match the template key in the command exactly,
              "reasoning": "<your reasoning for suggesting the command based on the context>"
            }}

            missing_fields = {{
              "key": string // what field you need from the user,
              "reasoning": string // why you need the field
            }}

            It is important to remember that for each missing field you have for a command an equivalent
            <key> should be there which is surrounded by <>. If that is not possible, do not return
            any missing fields.

            With all this said, here are some key instructions and points to remember:

            1) Feel free to assume that the user is lazy, so the prompts can be half-assed and
            incomplete. It is your job to infer what the user wants.
            2) The user is always going to be on macOS; some packages might not be installed. For
            those, chain the commands in the output, something like `brew install package && package
            doSomething` etc.
            3) Assume basic packages such as git, grep, http and other similar packages are installed.

            Here are some examples:

            example 1 ->
            user query - show me all the files
            output = {{
              "response": [
                {{
                  "cmd": "ls",
                  "missing_fields": [],
                  "reasoning": "the ls command shows all the files in the current directory"
                }},
                {{
                  "cmd": "ls <path>",
                  "missing_fields": [
                    {{
                      "key": "path",
                      "reasoning": "the user might want to see all files in a different directory"
                    }}
                  ],
                  "reasoning": "the ls command shows all the files in the current directory"
                }},
                {{
                  "cmd": "ls -la",
                  "missing_fields": [],
                  "reasoning": "the ls -la command shows all the files in the current directory as a
                  proper list with more information"
                }}
              ]
            }}

            example 2 ->
            user query - scp json
            output = {{
              "response": [
                {{
                  "cmd": "scp ./*.json root@172.186.26.5:/root/destination",
                  "missing_fields": [],
                  "reasoning": "the user did an scp command earlier, assuming they want to do the
                  same scp command, this time send all json from local to remote"
                }},
                {{
                  "cmd": "scp ./*.json user@172.186.26.5:<path>",
                  "missing_fields": [
                    {{
                      "key": "path",
                      "reasoning": "the user did the a scp command earlier, they might want to send
                      all json files to the same machine, but in a different directory"
                    }}
                  ],
                  "reasoning": "the user did an scp command earlier, assuming they want to do the
                  same scp command, this time send all json from local to remote"
                }}
              ]
            }}

            Also a side note - do not suggest commands which have a curly brace in them.

            That’s it for the examples; here is the context:
            {context}
        "#, context = ctx_string)
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ").to_string()
    }
}

// request/response common
#[derive(Serialize, Deserialize)]
pub struct OllamaMessage {
    pub role: String,
    pub content: String,
}

// request
#[derive(Serialize, Deserialize)]
pub struct OllamaRequest {
    pub model: String,
    pub format: String,
    pub stream: bool,
    pub messages: Vec<OllamaMessage>,
}

// response
#[derive(Serialize, Deserialize)]
pub struct OllamaResponse {
    pub model: String,
    pub created_at: String,
    pub message: OllamaMessage,
    pub done_reason: String,
    pub total_duration: u64,
    pub load_duration: u64,
    pub prompt_eval_count: u64,
    pub prompt_eval_duration: u64,
    pub eval_count: u64,
    pub eval_duration: u64,
}

#[derive(Serialize, Deserialize)]
pub struct OllamaPlaceholderResponse {
    pub response: Vec<ModelSuggestion>,
}

#[derive(Serialize, Deserialize)]
pub struct ModelSuggestion {
    pub cmd: String,
    pub reasoning: String,
    pub missing_fields: Vec<MissingField>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MissingField {
    pub key: String,
    pub reasoning: String,
}
