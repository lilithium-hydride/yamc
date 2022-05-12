use std::io::{stdout, Write};
use std::process::Command;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    queue,
    style::Print,
    terminal,
    Result,
};

struct Config {
    image_margins: [u16; 4],  // Top, left, bottom, right
    image_size: [u16; 2],
}

struct CleanUp;
impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode();
    }
}

fn print_image((config, cover_art): (Config, Vec<u8>)) {
    stdout().write("\r\n".repeat(config.image_margins[0] as usize).as_ref());
    let mut cover_art_iter = cover_art.split(|x| x == &b'\n').peekable();
    while let Some(line) = cover_art_iter.next() {
        stdout().write(" ".repeat(config.image_margins[1] as usize).as_bytes());
        stdout().write(line);
        if cover_art_iter.peek().is_some() {
            stdout().write("\r\n".as_bytes());
        }
    }
    stdout().write("\r\n".repeat(config.image_margins[2] as usize).as_ref());
}

fn image(config: Config) -> (Config, Vec<u8>) {
    let chafa_output = Command::new("chafa")
        .arg("")
        .arg("--format").arg("symbols")
        .arg("--stretch")
        .arg("--size").arg((config.image_size[0]).to_string().to_owned() + "x" + &(config.image_size[1]).to_string())
        .arg("--margin-bottom").arg((config.image_margins[2] + 1).to_string())
        .arg("--margin-right").arg((config.image_margins[3] + 2).to_string())
        .output().unwrap();
    let chafa_err = String::from_utf8_lossy(&chafa_output.stderr);
    if chafa_err != "" {
        println!("{}", chafa_err);
    }
    (config, chafa_output.stdout)
}

fn main() -> Result<()> {
    let _clean_up = CleanUp;
    let config = Config {
        image_margins: [1, 2, 1, 2],
        image_size: [20, 10],
    };

    let terminal_size = terminal::size().unwrap();
    terminal::enable_raw_mode()?;


    print_image(image(config));


    Ok(())
}
