mod config;

use crate::config::Config;


use std::{
    io::{stdout, Write},
    cmp,
    process,
    time::Duration,
    fs::File,
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
        MoveTo(config.image.margins.left + config.image.margins.right + config.image.size.0, 3),
        PrintStyledContent(metadata.artists().unwrap().join(", ").bold()),  // TODO: Verify that this works w/ supporting players
        PrintStyledContent(" - ".reset()),  // .reset() just applies zero formatting. Used instead of Print() for ease of future style modifications.
        PrintStyledContent(metadata.title().unwrap().reset()),
        Clear(ClearType::UntilNewLine),  // Get rid of any remaining text from a previous iteration
        MoveTo(config.image.margins.left + config.image.margins.right + config.image.size.0, 4),
        PrintStyledContent(metadata.album_name().unwrap().italic()),
        Clear(ClearType::UntilNewLine),
    );
    stdout.flush();
}

fn print_buttons(config: Config, status: PlaybackStatus) {
    let mut stdout = stdout();
    //  ⏮          ⏭ 
    queue!(stdout,
        MoveTo(config.image.margins.left + config.image.margins.right + config.image.size.0, 6),
        
        PrintStyledContent(config.controls_bar.cap_left.magenta()),
        
        PrintStyledContent((format!(
            "{}{}{}",
            " ".repeat((config.controls_bar.button_prev.margins.0 + config.controls_bar.button_prev.padding.0).into()),
            config.controls_bar.button_prev.icon,
            " ".repeat((config.controls_bar.button_prev.margins.1 + config.controls_bar.button_prev.padding.1).into()),
            )).black().on_magenta()),
        
        PrintStyledContent((format!(
            "{}{}{}",
            " ".repeat((config.controls_bar.button_playpause.margins.0 + config.controls_bar.button_playpause.padding.0).into()),
            match status {
                PlaybackStatus::Playing => config.controls_bar.button_playpause.icon_state1,
                PlaybackStatus::Paused|PlaybackStatus::Stopped => config.controls_bar.button_playpause.icon_state2,
            },
            " ".repeat((config.controls_bar.button_playpause.margins.1 + config.controls_bar.button_playpause.padding.1).into()),
            )).black().on_magenta()),
        
        PrintStyledContent((format!(
            "{}{}{}",
            " ".repeat((config.controls_bar.button_next.margins.0 + config.controls_bar.button_next.padding.0).into()),
            config.controls_bar.button_next.icon,
            " ".repeat((config.controls_bar.button_next.margins.1 + config.controls_bar.button_next.padding.1).into()),
            )).black().on_magenta()),
        
        PrintStyledContent(config.controls_bar.cap_right.magenta()),
    );
    
    stdout.flush();
}

fn print_image((config, cover_art): (Config, Vec<u8>)) {
    let mut stdout = stdout();
    
    queue!(stdout, 
        MoveTo(0, 0),
        Print("\r\n".repeat(config.image.margins.top as usize)),
    );
    
    let mut cover_art_iter = cover_art.split(|x| x == &b'\n').peekable();
    while let Some(line) = cover_art_iter.next() {
        queue!(stdout, 
            Print(" ".repeat(config.image.margins.left as usize)),
            Print(String::from_utf8_lossy(line)),
        );
        if cover_art_iter.peek().is_some() {
            stdout.queue(Print("\r\n"));
        }
    }
    
    stdout.queue(Print("\r\n".repeat(cmp::max(0, config.image.margins.bottom as usize - 1))));
}

fn image(config: Config, path: &str) -> (Config, Vec<u8>) {
    let chafa_output = process::Command::new("chafa")
        .arg(path)
        //.arg("--format").arg("symbols")
        .arg("--stretch")
        .arg("--size").arg((config.image.size.0).to_string() + "x" + &(config.image.size.1).to_string())
        .arg("--margin-bottom").arg((config.image.margins.bottom + 1).to_string())
        .arg("--margin-right").arg((config.image.margins.right + 2).to_string())
        .output().unwrap();
    let chafa_err = String::from_utf8_lossy(&chafa_output.stderr);
    if chafa_err != "" {
        println!("{}", chafa_err);
    }
    (config, chafa_output.stdout)
}

async fn print_all(config: Config, metadata: Metadata, status: PlaybackStatus) {
    match metadata.art_url() {
        Some(url) => print_image(image(config, url)),
        None => {
            // TODO: Try to rip art from the audio file. TianyiShi2001/audiotags may be able to do this without bringing in all of ffmpeg.
        }
    }
    print_metadata(config, metadata);
    print_buttons(config, status);
}

async fn handle_events(config: Config, player: Player<'_>) {
    let mut reader = EventStream::new();
    let mut stdout = stdout();
    
    loop {
        let mut delay = Delay::new(Duration::from_millis(100)).fuse();
        let mut terminal_event = reader.next().fuse();

        select! {
            _ = delay => {
                // TODO: Refresh only when track changes. See https://github.com/Mange/mpris-rs/blob/master/examples/events.rs
                async_std::task::spawn(print_all(config, player.get_metadata().unwrap(), player.get_playback_status().unwrap()));
            },
            maybe_event = terminal_event => {
                match maybe_event {
                    Some(Ok(Event::Mouse(event))) => match event.kind {
                        MouseEventKind::Down(btn) if btn == MouseButton::Left =>{
                            
                            let prev_lbound = config.image.margins.left + config.image.size.0 + config.image.margins.right + 1 + config.controls_bar.button_prev.margins.0;
                            let prev_ubound = prev_lbound + config.controls_bar.button_prev.padding.0 + 1 + config.controls_bar.button_prev.padding.1;
                            let pp_lbound = prev_ubound + config.controls_bar.button_prev.margins.1 + config.controls_bar.button_playpause.margins.0;
                            let pp_ubound = pp_lbound + config.controls_bar.button_playpause.padding.0 + 1 + config.controls_bar.button_playpause.padding.1;
                            let next_lbound = pp_ubound + config.controls_bar.button_playpause.margins.1 + config.controls_bar.button_next.margins.0;
                            let next_ubound = next_lbound + config.controls_bar.button_next.padding.0 + 1 + config.controls_bar.button_next.padding.1;
                            
                            if event.column >= prev_lbound && event.column < prev_ubound && event.row == 6{
                                player.previous();
                            } else if event.column >= pp_lbound && event.column < pp_ubound && event.row == 6 {
                                player.play_pause();
                            } else if event.column >= next_lbound && event.column < next_ubound && event.row == 6 {
                                player.next();
                            }
                        },
                        _ => (),
                    }
                    Some(Ok(Event::Key(event))) => {
                        match event.code {
                            KeyCode::Esc | KeyCode::Char('q') => break,
                            KeyCode::Left      => { player.previous(); },
                            KeyCode::Right     => { player.next(); },
                            KeyCode::Char(' ') => { player.play_pause(); },
                            KeyCode::Char('r') => {
                                stdout.execute(Clear(ClearType::All));
                                async_std::task::spawn(print_all(config, player.get_metadata().unwrap(), player.get_playback_status().unwrap()));
                            },
                            _ => {}
                        }
                    }
                    Some(Ok(Event::Resize(_, size_y))) => {}
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
    
    let config_path = format!("{}/yamc/yamc.ron", env!("XDG_CONFIG_HOME"));
    let config_file = File::open(&config_path).expect(&*format!("Failed to open configuration file at {}", config_path));
    let config: Config = match ron::de::from_reader(config_file) {
        Ok(x) => x,
        Err(e) => {
            println!("Failed to load settings from config: {}", e);
            process::exit(1);
        }
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
