use std::{
    collections::HashMap,
    fs::{self, File},
    path::Path,
    process,
};

use clap::Parser;
use serde::{Deserialize, Serialize};
use taskly::{
    cli::{Cli, Commands, TaskStatus},
    utils::{self, DirError, Dirs},
};
use time::{
    OffsetDateTime,
    format_description::{self},
};

#[derive(Deserialize, Serialize)]
struct TaskContainer {
    tasks: HashMap<u64, Task>,
}

#[derive(Deserialize, Serialize)]
struct Task {
    description: String,
    status: TaskStatus,
    #[serde(with = "time::serde::rfc3339")]
    created: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    updated: OffsetDateTime,
}

fn main() {
    let taskly_state = match utils::get_app_dir(Dirs::State) {
        Ok(p) => p,
        Err(DirError::DoesNotExist(path)) => {
            fs::create_dir_all(&path)
                .map_err(DirError::IoError)
                .expect("Failed to create taskly directory");
            path
        }
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
    let tasks_filepath = taskly_state.join("tasks.json");
    let id_filepath = taskly_state.join("next_id.txt");

    let time = OffsetDateTime::now_local().unwrap_or_else(|e| {
        eprintln!("Failed to get local time offset: {e}");
        println!("Falling back to UTC");
        OffsetDateTime::now_utc()
    });

    let mut container = if Path::new(&tasks_filepath).exists() {
        let tasks = fs::read_to_string(&tasks_filepath).expect("Failed to read tasks.json");
        serde_json::from_str(&tasks).expect("Failed to read json from string")
    } else {
        TaskContainer {
            tasks: HashMap::new(),
        }
    };

    let cli = Cli::parse();
    if let Some(cmd) = &cli.command {
        match cmd {
            Commands::Add { description } => {
                if !id_filepath.exists() {
                    fs::write(&id_filepath, "0").expect("Failed to initialise next_id.txt");
                }

                let id_string =
                    fs::read_to_string(&id_filepath).expect("Failed to read id file to string");

                let id = id_string
                    .parse::<u64>()
                    .expect("Failed to parse id string to u64");

                let new_id = id + 1;
                // NOTE: write creates a file if it does not exist, if it does exist it will
                // replace the contexts. Perfect.
                fs::write(&id_filepath, new_id.to_string())
                    .expect("Failed to write new id to next_id.txt");

                let task = Task {
                    description: description.into(),
                    status: TaskStatus::Todo,
                    created: time,
                    updated: time,
                };

                container.tasks.insert(new_id, task);

                if !tasks_filepath.exists() {
                    File::create(&tasks_filepath).expect("Failed to create tasks.json");
                }

                let json = serde_json::to_string_pretty(&container)
                    .expect("Failed to serialize container");

                if let Err(e) = fs::write(&tasks_filepath, json) {
                    eprintln!("Failed to write to tasks.json: {e:?}");
                };
            }
            Commands::Update { id, description } => {
                if !tasks_filepath.exists() {
                    println!("No tasks found, start create one first");
                    return;
                }

                let old_task = container.tasks.get(id).unwrap_or_else(|| {
                    println!("No task with found with ID: {id}");
                    process::exit(1);
                });

                let new_task = Task {
                    description: description.to_string(),
                    status: old_task.status.clone(),
                    created: old_task.created,
                    updated: time,
                };

                container.tasks.insert(*id, new_task);

                let json = serde_json::to_string_pretty(&container)
                    .expect("Failed to serialize container");

                if let Err(e) = fs::write(&tasks_filepath, json) {
                    eprintln!("Failed to write to tasks.json: {e:?}");
                };
            }
            Commands::Delete { id } => {
                if !tasks_filepath.exists() {
                    println!("No tasks found, start create one first");
                    return;
                }

                container
                    .tasks
                    .remove(id)
                    .expect("No task found with that ID");

                let json = serde_json::to_string_pretty(&container)
                    .expect("Failed to serialize container");

                if let Err(e) = fs::write(&tasks_filepath, json) {
                    eprintln!("Failed to write to tasks.json: {e:?}");
                };
            }
            Commands::List { status, all } => {
                if *all {
                    let tasks = container.tasks.iter().collect::<Vec<_>>();

                    list_tasks(&tasks);
                    return;
                }
                match status {
                    TaskStatus::Todo => {
                        let tasks = container
                            .tasks
                            .iter()
                            .filter(|(_, task)| task.status == TaskStatus::Todo)
                            .collect::<Vec<_>>();

                        list_tasks(&tasks);
                    }
                    TaskStatus::Complete => {
                        let tasks = container
                            .tasks
                            .iter()
                            .filter(|(_, task)| task.status == TaskStatus::Complete)
                            .collect::<Vec<_>>();

                        list_tasks(&tasks);
                    }
                    TaskStatus::Other(category) => {
                        let tasks = container
                            .tasks
                            .iter()
                            .filter(|(_, task)| {
                                task.status == TaskStatus::Other(category.to_string())
                            })
                            .collect::<Vec<_>>();

                        list_tasks(&tasks);
                    }
                }
            }
            Commands::Status { id, status } => {
                if !tasks_filepath.exists() {
                    println!("No tasks found, start create one first");
                    return;
                }

                let old_task = container.tasks.get(id).unwrap_or_else(|| {
                    println!("No task with found with ID: {id}");
                    process::exit(1);
                });

                let new_task = Task {
                    description: old_task.description.clone(),
                    status: status.clone(),
                    created: old_task.created,
                    updated: time,
                };

                container.tasks.insert(*id, new_task);

                let json = serde_json::to_string_pretty(&container)
                    .expect("Failed to serialize container");

                if let Err(e) = fs::write(&tasks_filepath, json) {
                    eprintln!("Failed to write to tasks.json: {e:?}");
                };
            }
        }
    }
}

fn list_tasks(tasks: &Vec<(&u64, &Task)>) {
    for (id, task) in tasks {
        println!("Id: {}", id);
        println!("Description: {}", task.description);
        println!("Status: {}", task.status);
        println!("Created: {}", format_time(task.created));
        println!("Updated: {}", format_time(task.updated));
        println!();
    }
}

fn format_time(time: OffsetDateTime) -> String {
    let format = format_description::parse(
        "[year].[month].[day] at [hour]:[minute]:[second] [offset_hour sign:mandatory]:[offset_minute]",
    ).expect("Failed parse format");

    time.format(&format).expect("Failed to format time")
}
