use serde_json::Value;
use std::io::Read;
use tokio::io;

use proc_macros::load_lua_script;
use crate::{App, LuaScripts};
use crate::Tokens::*;

pub fn LoadJson (fileName: &str) -> Result <Value, io::Error> {
    let mut file = std::fs::File::open(fileName)?;  // Open the file
    let mut fileContent = String::new();
    file.read_to_string(&mut fileContent)?;  // Read content into a string

    let json =
        serde_json::from_str( &fileContent )?;
    Ok(json)
}

fn FindLanguage (languageEnding: &str) -> Option<Languages> {
    for language in LANGS.iter() {
        if languageEnding == language.1 {
            return Some(language.0.clone());
        }
    } None
}

pub fn LoadSyntaxScripts(fileName: &str, syntaxHighlighting: &mut LuaScripts) -> io::Result<()> {
    // gathering the data
    let content = LoadJson(fileName)?;

    let (scripts, endings) = (
        content.get("scripts"),
        content.get("file_ending")
    );
    if scripts.is_none() || endings.is_none() { return Ok(()); }
    let (scripts, endings) = (scripts.unwrap().as_array(), endings.unwrap().as_array());
    if scripts.is_none() || endings.is_none() { return Ok(()); }
    let (scripts, endings) = (scripts.unwrap(), endings.unwrap());

    // loading the scripts
    for (i, script) in scripts.iter().enumerate() {
        let script = script.as_str().unwrap().to_string();  // is this right?
        let language = FindLanguage(endings[i].as_str().unwrap());
        if let Some(lang) = language {
            println!("Loading script \"{}\", {:?}", script, lang);
            load_lua_script!(
                syntaxHighlighting,
                lang,
                script
            );
        }
    } Ok(())
}


/*
// loads all tasks
fn loadTasks(fileName: &str) -> Vec <Task> {
    let mut loadedTasks: Vec <Task> = vec!();

    let mut file = std::fs::File::open(fileName).expect("Failed to open file");  // Open the file
    let mut fileContent = String::new();

    file.read_to_string(&mut fileContent).expect("Failed to read file");  // Read content into a string
    println!("Content: {}", fileContent);

    let json: serde_json::Value =
        serde_json::from_str( &fileContent ).expect("JSON was not well-formatted");

    // parsing the json into individual tasks
    let tasks = json.get("tasks").unwrap();

    // itterating through all elements
    let allTasks = tasks.as_array().expect("Failed to unpack array");
    for task in allTasks.iter() {
        // getting the full task
        let name = task.get("taskName").expect("Improper Json formatting");
        let description = task.get("taskDescription").expect("Improper Json formatting");
        let newTask = Task {
            taskName: name.to_string().trim().to_string(),
            taskDescription: description.to_string().trim().to_string(),
        };

        // adding the value
        loadedTasks.push(newTask);
    }

    return loadedTasks;
}


// saving a json file for the given tasks
fn SaveTasks(fileName: &str, tasks: &Vec <Task>) {
    // manually constructing the serialized data because for some reason everything seems broken and adds random \\\" s all around the place
    let mut innerVec: Vec <serde_json::Value> = vec!();
    for task in tasks
    {
        innerVec.push(
            serde_json::json!(  // the individual tasks
                {
                    "taskName": task.taskName,
                    "taskDescription": task.taskDescription
                }
            )
        );
    }

    // constructing the final json
    let jsonData = serde_json::json!(
        {
            "tasks": innerVec
    });

    let jsonString = serde_json::to_string_pretty(&jsonData).expect("Failed to serialize").replace("\\\"", "");

    println!("Output: {}", jsonString);

    let mut file = std::fs::File::create(fileName).expect("Failed to create file");
    file.write_all(jsonString.as_bytes()).expect("Failed to write to file");
}
 */