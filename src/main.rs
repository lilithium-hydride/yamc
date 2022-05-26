#![feature(iter_intersperse)]

use std::{
    io::{stdout, Write},
    cmp,
    process,
    time::Duration,
};

use futures::{future::FutureExt, select, StreamExt};
use futures_timer::Delay;

use crossterm::{
    cursor::{self, MoveTo}, 
    event::{
        Event, 
        EventStream,
        MouseButton,
        MouseEventKind,
        KeyCode, 
        DisableMouseCapture, 
        EnableMouseCapture,
    },
    execute, queue,
    ExecutableCommand, QueueableCommand,
    style::{Print, PrintStyledContent, Stylize},
    terminal::{
        self, 
        enable_raw_mode, 
        disable_raw_mode, 
        EnterAlternateScreen, 
        LeaveAlternateScreen, 
        DisableLineWrap, 
        EnableLineWrap,
        Clear,
        ClearType,
    },
    Result,
};

use mpris::{Metadata, PlaybackStatus, Player, PlayerFinder};


#[derive(Clone, Copy)]
struct Config {
    image_margins: [u16; 4],  // Top, left, bottom, right
    image_size: [u16; 2],
}

struct CleanUp;
impl Drop for CleanUp {
    fn drop(&mut self) {
        execute!(stdout(), 
            DisableMouseCapture,
        );
        disable_raw_mode();
        execute!(stdout(),
            EnableLineWrap,
            LeaveAlternateScreen,
            cursor::Show
        );
    }
}

fn print_metadata(config: Config, metadata: Metadata) {
    let mut stdout = stdout();
    
    queue!(stdout,
        MoveTo(config.image_margins[1] + config.image_margins[3] + config.image_size[0], 3),
        PrintStyledContent(metadata.artists().unwrap().iter().intersperse(&", ").map(|x| x.to_string()).collect::<String>().bold()),  // TODO: Verify that this works w/ supporting players
        PrintStyledContent(" - ".reset()),  // .reset() just applies zero formatting. Used instead of Print() for ease of future style modifications.
        PrintStyledContent(metadata.title().unwrap().reset()),
        Clear(ClearType::UntilNewLine),  // Get rid of any remaining text from a previous iteration
        MoveTo(config.image_margins[1] + config.image_margins[3] + config.image_size[0], 4),
        PrintStyledContent(metadata.album_name().unwrap().italic()),
        Clear(ClearType::UntilNewLine),
    );
    stdout.flush();
}

fn print_buttons(config: Config, status: PlaybackStatus) {
    let mut stdout = stdout();
    
    queue!(stdout,
        MoveTo(config.image_margins[1] + config.image_margins[3] + config.image_size[0], 6),
        PrintStyledContent("".magenta()),
        PrintStyledContent(" ⏮     ".black().on_magenta()),
        PrintStyledContent(match status {
                PlaybackStatus::Playing => "",
                PlaybackStatus::Paused|PlaybackStatus::Stopped => ""
        }.black().on_magenta()),
        PrintStyledContent("     ⏭ ".black().on_magenta()),
        PrintStyledContent("".magenta()),
    );
    
    stdout.flush();
}

fn print_image((config, cover_art): (Config, Vec<u8>)) {
    let mut stdout = stdout();
    
    queue!(stdout, 
        MoveTo(0, 0),
        Print("\r\n".repeat(config.image_margins[0] as usize)),
    );
    
    let mut cover_art_iter = cover_art.split(|x| x == &b'\n').peekable();
    while let Some(line) = cover_art_iter.next() {
        queue!(stdout, 
            Print(" ".repeat(config.image_margins[1] as usize)),
            Print(String::from_utf8_lossy(line)),
        );
        if cover_art_iter.peek().is_some() {
            stdout.queue(Print("\r\n"));
        }
    }
    
    stdout.queue(Print("\r\n".repeat(cmp::max(0, config.image_margins[2] as usize - 1))));
}

fn image(config: Config, path: &str) -> (Config, Vec<u8>) {
    let chafa_output = process::Command::new("chafa")
        .arg(path)
        .arg("--format").arg("symbols")
        .arg("--stretch")
        .arg("--size").arg((config.image_size[0]).to_string() + "x" + &(config.image_size[1]).to_string())
        .arg("--margin-bottom").arg((config.image_margins[2] + 1).to_string())
        .arg("--margin-right").arg((config.image_margins[3] + 2).to_string())
        .output().unwrap();
    let chafa_err = String::from_utf8_lossy(&chafa_output.stderr);
    if chafa_err != "" {
        println!("{}", chafa_err);
    }
    (config, chafa_output.stdout)
}

async fn print_all(config: Config, metadata: Metadata, status: PlaybackStatus) {

    print_image(image(config, metadata.art_url().unwrap()));

    print_metadata(config, metadata);

    print_buttons(config, status);
}

async fn handle_events(config: Config, player: Player<'_>) {
    let mut reader = EventStream::new();

    loop {
        let mut delay = Delay::new(Duration::from_millis(100)).fuse();  // Don't wait to run the first time
        let mut terminal_event = reader.next().fuse();


        select! {
            _ = delay => {
                // TODO: Refresh only when track changes. See https://github.com/Mange/mpris-rs/blob/master/examples/events.rs
                async_std::task::spawn(print_all(config, player.get_metadata().unwrap(), player.get_playback_status().unwrap()));
            },
            maybe_event = terminal_event => {
                match maybe_event {
                    Some(Ok(Event::Mouse(mouse_event))) => match mouse_event.kind {
                        MouseEventKind::Down(btn) if btn == MouseButton::Left => {
                            if mouse_event.column >= 30 && mouse_event.column <= 32 && mouse_event.row == 6 {
                                player.previous();
                            } else if mouse_event.column == 37 && mouse_event.row == 6 {
                                player.play_pause();
                            } else if mouse_event.column >= 42 && mouse_event.column <= 44 && mouse_event.row == 6 {
                                player.next();
                            }
                        },
                        //MouseEventKind::ScrollDown => (),
                        //MouseEventKind::ScrollUp => (),
                        _ => (),
                    }
                    Some(Ok(terminal_event)) => {
                        if terminal_event == Event::Key(KeyCode::Esc.into()) || terminal_event == Event::Key(KeyCode::Char('q').into()) {
                            break;
                        } else if terminal_event == Event::Key(KeyCode::Left.into()) {
							player.previous();
						} else if terminal_event == Event::Key(KeyCode::Right.into()) {
                            player.next();
						} else if terminal_event == Event::Key(KeyCode::Char(' ').into()) {
                            player.play_pause();
						}
                    }
                    Some(Err(e)) => println!("Error: {:?}\r", e),
                    None => break,
                }
            }
        };
    }
}

#[async_std::main]
async fn main() -> Result<()> {
    let _clean_up = CleanUp;
    let config = Config {
        image_margins: [1, 2, 1, 3],
        image_size: [24, 12],
    };

    let player_stuff = async {
        let player = PlayerFinder::new().expect("Failed to connect to D-Bus")
            .find_active().expect("Failed to find a player");
        let metadata = player.get_metadata().expect("Failed to get player metadata");
        (player, metadata)
    };

    execute!(stdout(), 
        cursor::Hide,
        EnterAlternateScreen, 
        EnableMouseCapture, 
        DisableLineWrap, 
    )?;
    enable_raw_mode()?;


    let (player, metadata) = player_stuff.await;
    async_std::task::spawn(print_all(config, metadata, player.get_playback_status().unwrap()));
    
    async_std::task::block_on(handle_events(config, player));
    
    Ok(())
}
