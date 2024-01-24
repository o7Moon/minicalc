use crossterm::event::KeyCode;
use crossterm::terminal;
use crossterm::cursor;
use crossterm::queue;
use crossterm::event;
use crossterm::style::Print;
use crate::minicalc::State;
use crate::Args;
use std::io::Write;
use std::io::stdout;
use std::time::Duration;

pub fn crossterm_main(args: Args) {
    let mut state = State::default();
    state.vars_path = args.vars;
    state.read_vars();
    
    if terminal::enable_raw_mode().is_err() {println!("terminal does not support raw mode, cant run term frontend"); return};
    
    _ = queue!(stdout(),
        terminal::DisableLineWrap,
        cursor::SavePosition,
        cursor::EnableBlinking,
        Print(state.display()),
    );

    _ = stdout().flush();

    loop {
        _ = queue!(stdout(),
            cursor::RestorePosition,
            terminal::Clear {0: terminal::ClearType::CurrentLine}
        );
        
        if event::poll(Duration::from_millis(1000)).unwrap_or(false) {
            match event::read().ok() {
                Some(event) => {
                    match event {
                        event::Event::Key(event) => {
                            match event.code {
                                KeyCode::Char(char) => 'char_case: {
                                    if char == ':' && state.command.is_none() {
                                        state.enter_command_entry("".to_owned());
                                        break 'char_case;
                                    }
                                    state.type_string(char.to_string())
                                },
                                KeyCode::Backspace => {
                                    state.delete_one();
                                    state.cached_equation_display = None;
                                },
                                KeyCode::Enter => {
                                    if state.command.is_some() {
                                        execute_command(&mut state)
                                    } else {
                                        state.equation.eval_mut();
                                        state.cached_equation_display = None;
                                    }
                                },
                                KeyCode::Esc => {
                                    if state.command.is_none() {
                                        state.enter_equation_entry()
                                    }
                                },
                                _ => {}
                            }
                        },
                        _ => {}
                    }
                },
                None => {}
            }
        }
        _ = queue!(stdout(),
            Print(state.display())
        );

        _ = stdout().flush();
    }
}

fn execute_command(state: &mut State) {
    // extra logic
    state.execute_command()
}
