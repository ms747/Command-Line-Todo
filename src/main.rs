use clap::{App, Arg};
use prettytable::{cell, format, row, Table};
use rusqlite::{params, Connection, Result};
use std::io::prelude::*;
use std::{
    env::{temp_dir, var},
    fs::File,
    io::Read,
    process::Command,
};
use uuid::Uuid;

#[derive(Debug)]
struct Task {
    id: u32,
    description: String,
}

#[allow(dead_code)]
fn print_tasks(conn: &rusqlite::Connection) -> Result<()> {
    let mut tasks_querry = conn.prepare("SELECT * from tasks")?;
    let tasks_iter = tasks_querry.query_map(params![], |row| {
        Ok(Task {
            id: row.get(0)?,
            description: row.get(1)?,
        })
    })?;

    let tasks = tasks_iter.collect::<Vec<_>>();

    if tasks.len() == 0 {
        println!("No Tasks");
        return Ok(());
    }

    let mut task_table = Table::new();
    task_table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    task_table.set_titles(row!["ID", "Description"]);

    for task in tasks {
        let temp_task = task.unwrap();
        task_table.add_row(row![temp_task.id, temp_task.description]);
    }

    task_table.printstd();

    Ok(())
}

#[allow(dead_code)]
fn add_task(conn: &rusqlite::Connection, description: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO tasks (description) VALUES (?1)",
        params![description],
    )?;
    Ok(())
}

#[allow(dead_code)]
fn delete_task(conn: &rusqlite::Connection, id: u32) -> Result<()> {
    conn.execute("DELETE FROM tasks where id=?1", params![id])?;
    Ok(())
}

#[allow(dead_code)]
fn update_task(conn: &rusqlite::Connection, id: u32, description: &str) -> Result<()> {
    conn.execute(
        "UPDATE tasks SET description=?2 WHERE id=?1",
        params![id, description],
    )?;
    Ok(())
}

fn update(conn: &rusqlite::Connection, id: u32) -> Result<()> {
    let mut tasks_querry = conn.prepare("SELECT * from tasks where id = ?")?;
    let tasks_iter = tasks_querry.query_map(params![id], |row| {
        Ok(Task {
            id: row.get(0)?,
            description: row.get(1)?,
        })
    })?;

    let tasks = tasks_iter.collect::<Vec<_>>();

    if tasks.len() == 0 {
        println!("No Tasks");
        return Ok(());
    }

    let mut content = String::new();

    for task in tasks {
        let temp_task = task.unwrap();
        content.push_str(temp_task.description.as_str());
    }

    let editor = var("EDITOR").expect("$EDITOR variable not set");
    let mut file_path = temp_dir();
    let uuid = Uuid::new_v4();
    file_path.push(uuid.to_string());
    let mut file = File::create(&file_path).expect("unable to create file");

    file.write_all(content.as_bytes())
        .expect("Unable to set buffer");

    Command::new(editor)
        .arg(&file_path)
        .status()
        .expect("Something went wrong");

    let mut editable = String::new();

    File::open(file_path)
        .expect("Cound not open file")
        .read_to_string(&mut editable)
        .expect("Failed to write buffer");

    update_task(&conn, id, editable.as_str())?;
    Ok(())
}

#[allow(dead_code)]
fn main() -> Result<()> {
    let mut db_path = dirs::home_dir().unwrap();
    db_path.push(".config");
    db_path.push("task");
    db_path.push("tasks");
    db_path.set_extension("db");
    let conn = Connection::open(db_path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks(
            id INTEGER PRIMARY KEY,
            description TEXT NOT NULL
        )",
        params![],
    )?;

    let matches = App::new("Tasks")
        .version("0.1")
        .author("Mayur S. <mayur.shah.ha@gmail.com>")
        .about("Command Line Todo List")
        .arg(
            Arg::with_name("ls")
                .help("List all tasks")
                .short('l')
                .long("ls"),
        )
        .arg(
            Arg::with_name("add")
                .help("Add to tasks")
                .short('a')
                .long("add")
                .value_name("TASK"),
        )
        .arg(
            Arg::with_name("update")
                .help("Update task for given id")
                .short('u')
                .long("udpate")
                .value_names(&["ID"]),
        )
        .arg(
            Arg::with_name("delete")
                .help("Delete task for given id")
                .short('d')
                .long("delete")
                .value_name("ID"),
        )
        .get_matches();

    // List
    if matches.is_present("ls") {
        print_tasks(&conn)?;
    }

    // Add
    if let Some(value) = matches.value_of("add") {
        add_task(&conn, value)?;
    }

    // Update
    if let Some(value) = matches.values_of("update") {
        let args = value.collect::<Vec<_>>();
        let id = args[0].parse::<u32>().unwrap();
        update(&conn, id)?;
        //let description = args[1];
        //update_task(&conn, id, description)?;
    }

    // Delete
    if let Some(value) = matches.value_of("delete") {
        let id = value.parse::<u32>().unwrap();
        delete_task(&conn, id)?;
    }

    Ok(())
}
