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
    // first iteration of the system prompt
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

    pub fn get_system_prompt_2(ctx: &Context) -> String {
        let ctx_string = to_string(ctx).unwrap_or_else(|e| panic!("{}", e));
        format!(
            r#"You are a command-line assistant. Your job is to provide up to 5 precise command suggestions based on context, with each suggestion in JSON format, adhering strictly to the schema below.

                Context provided:
                - User's current working directory and file types.
                - Past command history, including execution directory and timestamps.
                - Not all context will be relevant, as users may want new commands beyond their history.

                Guidelines for Suggestions:
                - Each suggestion has:
                    1. "reasoning": Brief explanation of the suggestion's purpose.
                    2. "commands": A list of commands, where:
                        - Commands can form a "chain" or be standalone.
                        - Use "<missing_field>" to indicate fields that the user must provide (e.g., `<branch_name>` in `git checkout -b <branch_name>`).
                        - Include "missing_fields" with:
                            - "key": Name of the missing field.
                            - "reasoning": Explanation for user input.
                            - "suggestions": Optional field with structured suggestions for missing fields, each with "value" and "reasoning".
                - **JSON Format**: Responses **must** strictly follow this JSON schema, facilitating easy parsing and user input handling:

                Example:
                ```json
                {{
                  "response": [
                    {{
                      "reasoning": "Single git push command assuming prior add/commit.",
                      "commands": [
                        {{
                          "cmd": "git push",
                          "missing_fields": [],
                          "reasoning": "Only push since changes have already been added and committed."
                        }}
                      ]
                    }},
                    {{
                      "reasoning": "Chain for adding, committing, and pushing changes.",
                      "commands": [
                        {{
                          "cmd": "git add <files>",
                          "missing_fields": [
                            {{
                              "key": "files",
                              "reasoning": "Specify files to add.",
                              "suggestions": [
                                {{"value": "-A", "reasoning": "Add all changed files"}},
                                {{"value": ".", "reasoning": "Add all files in current dir"}}
                              ]
                            }}
                          ],
                          "reasoning": "Add files before committing and pushing."
                        }},
                        {{
                          "cmd": "git commit -m <message>",
                          "missing_fields": [
                            {{
                              "key": "message",
                              "reasoning": "Specify a commit message.",
                              "suggestions": [
                                {{"value": "nits", "reasoning": "Quick message for minor fixes."}}
                              ]
                            }}
                          ],
                          "reasoning": "Commit changes with message before pushing."
                        }},
                        {{
                          "cmd": "git push",
                          "missing_fields": [],
                          "reasoning": "Push changes to remote repository."
                        }}
                      ]
                    }}
                  ]
                }}

    // second iteration of the system prompt, this tells the model to include flows as well
    pub fn get_system_prompt_2(ctx: &Context) -> String {
        let ctx_string = to_string(ctx).unwrap_or_else(|e| panic!("{}", e));
        format!(
            r#" You are a command line assistant and your job is to help the user find the right command or set of
            commands which they will then execute on their terminal. To help you out with this, I will provide
            you with some context such as what the users current working directory is, what are the files in their current working
            directory, for each file you'll also have information on what the type of the file is (directory or file).
            I will also provide you with a list of commands which they have executed in the past - this list
            will also have context for each command such as which directory the command was executed in and when it was executed.

            The context provided won't always be helpful since sometimes the user would want to execute a command which
            they haven't done so in the past.

            You will use this context and provide upto 5 suggestion for the user. A single suggestion can be of 2 types
            "chain" and "not chain"
            A chain suggestion is a series of commands which if the user runs will solve their issue. Following are
            some examples of chains
                git add -A; git commit -m <message>; git push -> this can be made into a chain of 3 commands
                    command 1 -> git add -A
                    command 2 -> git commit -m <message>
                    command 3 -> git push
                cd ../ && nvim foo.txt -> this can be broken into a chain of 2 commands
                    command 1 -> cd ../
                    command 2 -> nvim foo.txt

            In the above git example, as you see there is a missing field as well in the git commit -m <message> command. What this
            means is that the message field is something which you think should be provided by the user.
            You will keep track of each missing field in a command and present them to me as well. Each missing field has to be
            surrounded by angular brackets in following format <{{missing_field}}>
            If you think you have a suggestion for a missing field do provide that as well.

            Your suggestions should always be given in json format, this is very important as it will then be parsed by code
            to actually take in the user input if needed and to execute the command. Your output will always be of the following
            format

            if the user query is -> push to git
            output = {{
              "response": [
                    {{
                        "reasoning" : "this is the command for git push"
                        "commands" : [
                            {{
                                "cmd": "git push",
                                "missing_fields": [],
                                "reasoning": "assuming the user has already added and commited the changes, all we need to do is push them"
                            }}
                        ]
                    }},
                    {{
                        "reasoning" : "chain of commands to do a git add, commit and push, add some details here of why you decided to go with this chain"
                        "commands" : [
                            {{
                                "cmd": "git add <files>",
                                "missing_fields": [
                                    {{
                                         "key": "files",
                                         "reasoning": "the user needs to specify what file they want to add"
                                         "suggestions" : [
                                            {{
                                                "value" : "-A"
                                                "reasoning" : "the -A flag adds all files which have been changed"
                                            }},
                                            {{
                                                "value" : "."
                                                "reasoning" : "the . adds all files in this dir"
                                            }}
                                         ]
                                    }}
                                ],
                                "reasoning": "we need to add the files before we do the git push"
                            }},
                            {{
                                "cmd": "git commit -m <message>",
                                "missing_fields": [
                                    {{
                                         "key": "message", // it is very important that this key is present in the command enclosed in angulare braces
                                         "reasoning": "the user needs to specify what is it that they did in this commit"
                                         "suggestions" : [
                                            {{
                                                "value" : "nits"
                                                "reasoning" : "the user might have fixed some nits"
                                            }},
                                         ]
                                    }}
                                ],
                                "reasoning": "before we actually push the changes on to git, we need to do a git commit"
                            }},
                            {{
                                "cmd": "git push",
                                "missing_fields": [],
                                "reasoning": "pushing the changes onto git"
                            }}
                        ]
                    }}
              ]
            }}

            Notice in the above response commands is always a list of commands, I will infer from the list size whether or not it is a chain. If there is no missing
            field in a command, just return an empty list.
            Follow the schema exactly as given above.

            Another example which I'll just talk through is if the user says to scp all json files. What you can do in that case is check the users
            history for any other scp or ssh command and use that to suggest commands.

            The missing field should always be enclosed in angular brackets like the following
            <missing_field> please do not forget this

            When your command has files or username/password or ip addresses or anything specific like that
            always provide a missing field, and if you think you have a suggestion for that specific field
            provide it in the suggestions list of that missing field.

            Avoid outputting the same command again and again, if all that has changed is the missing field, in that case
            provide the command with a missing field and provide suggestions over there.b

            You can make the following assumptions:
                1) The user is lazy, so the prompts can be half-assed and incomplete. It is your job to infer what the user wants.
                2) Assume the user has access to basic packages on their machine such as git, grep, curl, vim, etc

            Only give a suggestion if your reasoning for that suggestion is strong and firm, if you think that a particular suggestion
            might or might not work, then it is fine to not give that suggestion.
            {context}
        "#, context = ctx_string)
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ").to_string()
    }
}

pub struct DummyResponse;
impl DummyResponse {
    pub fn get_dummy_response() -> String {
        r#"{"model":"qwen2.5","created_at":"2024-11-04T06:13:34.765879Z","message":{"role":"assistant","content":"{ \"response\": [ { \"reasoning\" : \"The user might want to create a new branch or switch to an existing one in their RustoverProjects directory.\", \"commands\" : [ { \"cmd\": \"git checkout -b <branch_name>\", \"missing_fields\": [ { \"key\": \"branch_name\", \"reasoning\": \"The user needs to specify the name of the branch they want to create or switch to.\" }, { \"suggestions\": [ { \"value\": \"feature_12345\", \"reasoning\": \"Example of a common branch naming convention for feature development.\" } ] } ], \"reasoning\": \"Switching to or creating a new branch is a common operation when working with Git repositories.\" } ] }, { \"reasoning\" : \"The user might want to create a new directory or move files around in the RustoverProjects directory.\", \"commands\" : [ { \"cmd\": \"mkdir <directory_name>\", \"missing_fields\": [ { \"key\": \"directory_name\", \"reasoning\": \"The user needs to specify the name of the new directory.\" }, { \"suggestions\": [ { \"value\": \"new_module\", \"reasoning\": \"Creating a new module within the project structure is a common task.\" } ] } ], \"reasoning\": \"Creating directories helps organize code better and follows good development practices.\" } ] }, { \"reasoning\" : \"The user might want to run tests or perform checks on the Rust code in this project.\", \"commands\" : [ { \"cmd\": \"cargo test\", \"missing_fields\": [], \"reasoning\": \"Running tests is a common operation after making changes to Rust code to ensure functionality.\" } ] }, { \"reasoning\" : \"The user might want to commit changes and push them to the repository.\", \"commands\" : [ { \"cmd\": \"git add .; git commit -m '<commit_message>'\", \"missing_fields\": [ { \"key\": \"commit_message\", \"reasoning\": \"The user needs to provide a commit message for their changes.\" }, { \"suggestions\": [ { \"value\": \"Add initial implementation of feature_12345\", \"reasoning\": \"Example commit message based on the branch name.\" } ] } ], \"reasoning\": \"Committing and pushing changes is a standard part of version control workflows.\" } ] } ] }"},"done_reason":"stop","total_duration":54367747041,"load_duration":36362250,"prompt_eval_count":1834,"prompt_eval_duration":2014534000,"eval_count":491,"eval_duration":52294908000}"#.to_string()
    }
}

// request/response common
#[derive(Serialize, Deserialize, Clone)]
pub struct OllamaMessage {
    pub role: String,
    pub content: String,
}

// request
#[derive(Serialize, Deserialize, Clone)]
pub struct OllamaRequest {
    pub model: String,
    pub format: String,
    pub stream: bool,
    pub messages: Vec<OllamaMessage>,
}

// response
#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
pub struct OllamaPlaceholderResponse {
    pub response: Vec<ModelSuggestion>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ModelSuggestion {
    pub reasoning: String,
    pub commands: Vec<SuggestedCommand>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SuggestedCommand {
    pub reasoning: String,
    pub cmd: String,
    pub missing_fields: Vec<MissingField>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MissingField {
    pub key: String,
    pub reasoning: String,
    pub suggestions: Vec<MissingFieldSuggestion>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MissingFieldSuggestion {
    pub value: String,
    pub reasoning: String,
}
