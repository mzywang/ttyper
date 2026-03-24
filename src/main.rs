mod cli;
mod components;
mod config;
mod error;
mod messages;
mod model;
mod resources;
mod types;

use clap::CommandFactory;
use clap::Parser;
use clap_complete::generate;
use std::io;
use std::time::Duration;
use tuirealm::{
    event::NoUserEvent, terminal::TerminalBridge, Application, EventListenerCfg, PollStrategy,
    Update,
};

use cli::{Command, Opt};
use components::test::TestComponent;
use error::TtyperError;
use messages::Msg;
use model::{Id, Model};
use types::Test;

fn main() -> eyre::Result<()> {
    let opt = Opt::parse();

    if let Some(Command::Completions { shell }) = opt.command {
        generate(shell, &mut Opt::command(), "ttyper", &mut io::stdout());
        return Ok(());
    }

    if opt.list_languages {
        opt.languages().map_err(TtyperError::Io)?.for_each(|name| {
            if let Some(s) = name.to_str() {
                println!("{}", s);
            }
        });

        return Ok(());
    }

    let config = opt.config();
    let contents = opt.gen_contents().ok_or_else(|| {
        TtyperError::Content(
            "Couldn't get test contents. Make sure the specified language actually exists.".into(),
        )
    })?;

    if contents.is_empty() {
        return Err(eyre::eyre!("Error: the provided file or language contains no words to type. If you specified a file, make sure it isn't empty."));
    }

    let mut terminal_bridge =
        TerminalBridge::init_crossterm().map_err(|e| TtyperError::Terminal(e.to_string()))?;

    let mut app: Application<Id, Msg, NoUserEvent> = Application::init(
        EventListenerCfg::<NoUserEvent>::default()
            .crossterm_input_listener(Duration::from_millis(20), 1)
            .poll_timeout(Duration::from_millis(10))
            .tick_interval(Duration::from_secs(1)),
    );

    app.mount(
        Id::Test,
        Box::new(TestComponent {
            test: Test::new(
                contents,
                !opt.no_backtrack,
                opt.sudden_death,
                !opt.no_backspace,
            ),
            theme: config.theme.clone(),
        }),
        vec![],
    )
    .map_err(|e| TtyperError::Application(e.to_string()))?;

    // Enable focus
    app.active(&Id::Test)
        .map_err(|e| TtyperError::Application(e.to_string()))?;

    let mut model = Model::new(app, terminal_bridge, config, opt);

    // Terminal initialization (Alternate screen, Raw mode) is already handled by
    let _ = model.terminal.clear_screen();

    while !model.quit {
        // Tick
        match model.app.tick(PollStrategy::Once) {
            Err(err) => {
                let _ = model.terminal.restore();
                return Err(TtyperError::Application(err.to_string()).into());
            }
            Ok(messages) => {
                if !messages.is_empty() {
                    model.redraw = true;
                    for msg in messages.into_iter() {
                        let mut msg = Some(msg);
                        while msg.is_some() {
                            msg = model.update(msg);
                        }
                    }
                }
            }
        }

        // Redraw
        if model.redraw {
            model.view()?;
            model.redraw = false;
        }
    }

    // Restore terminal
    let _ = model.terminal.restore();

    Ok(())
}
