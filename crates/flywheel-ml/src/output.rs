use serde::Serialize;

pub enum OutputFormat {
    Table,
    Json,
    Yaml,
}

pub fn print_table<T: Serialize>(items: &[T], headers: &[&str]) {
    // Print header
    for (i, h) in headers.iter().enumerate() {
        if i > 0 {
            print!("\t");
        }
        print!("{}", h);
    }
    println!();

    // Print separator
    for (i, _) in headers.iter().enumerate() {
        if i > 0 {
            print!("\t");
        }
        print!("--------");
    }
    println!();
}

pub fn print_json<T: Serialize>(item: &T) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(item)?;
    println!("{}", json);
    Ok(())
}

pub fn print_yaml<T: Serialize>(item: &T) -> anyhow::Result<()> {
    let yaml = serde_yaml::to_string(item)?;
    println!("{}", yaml);
    Ok(())
}
