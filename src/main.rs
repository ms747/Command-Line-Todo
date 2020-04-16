use prettytable::{cell, format, row, Table};
use rusqlite::{params, Connection, Result};

#[derive(Debug)]
struct Task {
    id: u32,
    description: String,
}

fn print_tasks(conn: &rusqlite::Connection) -> Result<()> {
    let mut tasks = conn.prepare("SELECT * from tasks")?;
    let tasks_iter = tasks.query_map(params![], |row| {
        Ok(Task {
            id: row.get(0)?,
            description: row.get(1)?,
        })
    })?;

    let mut task_table = Table::new();
    task_table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    task_table.set_titles(row!["ID", "Description"]);
    for task in tasks_iter {
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

fn main() -> Result<()> {
    let conn = Connection::open("./tasks.sqlite")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks(
            id INTEGER PRIMARY KEY,
            description TEXT NOT NULL
        )",
        params![],
    )?;

    let args = std::env::args().skip(1).collect::<Vec<String>>();

    if args.len() < 1 || args.len() > 3 {
        eprintln!("USAGE : task <COMMAND> [args...]");
        eprintln!("COMMANDS : ");
        eprintln!("add <task>");
        eprintln!("ls");
        eprintln!("del <id>");
        eprintln!("update <id> <description>");
        std::process::exit(1);
    }
    match args[0].as_str() {
        "add" => {
            if args.len() == 2 {
                add_task(&conn, &args[1])?
            } else {
                eprintln!("USAGE : add <task>");
            }
        }
        "ls" => {
            if args.len() == 1 {
                print_tasks(&conn)?
            } else {
                eprintln!("USAGE : ls");
            }
        }
        "del" => {
            if args.len() == 2 {
                delete_task(&conn, args[1].parse::<u32>().unwrap())?
            } else {
                eprintln!("USAGE : del <id>");
            }
        }
        "update" => {
            if args.len() == 3 {
                update_task(&conn, args[1].parse::<u32>().unwrap(), &args[2])?
            } else {
                eprintln!("USAGE : update <id> <description>");
            }
        }
        _ => println!("Feature Not Implemented"),
    };

    Ok(())
}
