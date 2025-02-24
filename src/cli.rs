use std::{fmt::Display, str::FromStr};

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum TaskStatus {
    Todo,
    Complete,
    Other(String),
}

impl FromStr for TaskStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str().trim() {
            "todo" => Ok(TaskStatus::Todo),
            "complete" => Ok(TaskStatus::Complete),
            other => Ok(TaskStatus::Other(other.to_string())),
        }
    }
}

impl Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Todo => write!(f, "Todo"),
            TaskStatus::Complete => write!(f, "Complete"),
            TaskStatus::Other(other) => write!(f, "{other}"),
        }
    }
}

#[derive(Parser)]
#[command(name = "Taskly", version = "0.1.0", about = "Manage tasks", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Commands {
    #[command(about = "Create task")]
    Add {
        #[arg()]
        description: String,
    },
    #[command(about = "Update task")]
    Update {
        #[arg()]
        id: u64,

        #[arg()]
        description: String,
    },
    #[command(about = "Delete task")]
    Delete {
        #[arg()]
        id: u64,
    },
    #[command(about = "List tasks")]
    List {
        #[arg(required = false, default_value_t = TaskStatus::Todo)]
        status: TaskStatus,

        #[arg(short, long)]
        all: bool,
    },
    #[command(about = "Mark task as finished/to-do")]
    Status {
        #[arg()]
        id: u64,

        #[arg()]
        status: TaskStatus,
    },
}
