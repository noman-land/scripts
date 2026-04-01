use std::fs;
use std::io;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};

#[derive(Debug, Clone)]
struct Utility {
    name: String,
    executable_path: PathBuf,
    installed: bool,
    selected: bool,
    initially_installed: bool,
}

impl Utility {
    fn is_installed(&self, install_dir: &Path) -> bool {
        let dest = install_dir.join(&self.name);
        if dest.is_symlink() {
            if let Ok(target) = fs::read_link(&dest) {
                return target == self.executable_path;
            }
        }
        false
    }

    fn install(&self, install_dir: &Path) -> io::Result<()> {
        let dest = install_dir.join(&self.name);
        if dest.exists() || dest.is_symlink() {
            fs::remove_file(&dest)?;
        }
        unix_fs::symlink(&self.executable_path, &dest)?;
        Ok(())
    }

    fn uninstall(&self, install_dir: &Path) -> io::Result<()> {
        let dest = install_dir.join(&self.name);
        if dest.is_symlink() {
            if let Ok(target) = fs::read_link(&dest) {
                if target == self.executable_path {
                    fs::remove_file(&dest)?;
                    return Ok(());
                }
            }
        }
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Not a valid installation: {}", self.name),
        ))
    }
}

fn discover_utilities(script_dir: &Path) -> Vec<Utility> {
    let mut utilities = Vec::new();

    if let Ok(entries) = fs::read_dir(script_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let name = match path.file_name().and_then(|n| n.to_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };

            if name.starts_with('.') {
                continue;
            }

            let executable = path.join(&name);
            if executable.is_file() && is_executable(&executable) {
                utilities.push(Utility {
                    name,
                    executable_path: executable,
                    installed: false,
                    selected: false,
                    initially_installed: false,
                });
            }
        }
    }

    utilities.sort_by(|a, b| a.name.cmp(&b.name));
    utilities
}

fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(metadata) = fs::metadata(path) {
        let perms = metadata.permissions();
        perms.mode() & 0o111 != 0
    } else {
        false
    }
}

fn update_install_status(utilities: &mut Vec<Utility>, install_dir: &Path) {
    for util in utilities.iter_mut() {
        util.installed = util.is_installed(install_dir);
        util.selected = util.installed;
        util.initially_installed = util.installed;
    }
}

fn apply_selections(utilities: &[Utility], install_dir: &Path) -> Vec<(String, String)> {
    let mut results = Vec::new();

    for util in utilities {
        let want_installed = util.selected;

        let result = if want_installed && !util.installed {
            let dest = install_dir.join(&util.name);
            match util.install(install_dir) {
                Ok(()) => format!("Installed: {} -> {}", util.name, dest.display()),
                Err(e) => format!("Failed to install {}: {}", util.name, e),
            }
        } else if !want_installed && util.installed {
            let dest = install_dir.join(&util.name);
            match util.uninstall(install_dir) {
                Ok(()) => format!("Uninstalled: {}", dest.display()),
                Err(e) => format!("Failed to uninstall {}: {}", util.name, e),
            }
        } else {
            continue;
        };
        results.push((util.name.clone(), result));
    }

    results
}

struct App {
    utilities: Vec<Utility>,
    list_state: ListState,
    install_dir: PathBuf,
    quit: bool,
    results: Vec<String>,
}

impl App {
    fn new(utilities: Vec<Utility>, install_dir: PathBuf) -> Self {
        let mut list_state = ListState::default();
        if !utilities.is_empty() {
            list_state.select(Some(0));
        }
        Self {
            utilities,
            list_state,
            install_dir,
            quit: false,
            results: Vec::new(),
        }
    }

    fn next(&mut self) {
        if self.utilities.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.utilities.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn previous(&mut self) {
        if self.utilities.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.utilities.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn toggle_selection(&mut self) {
        if let Some(i) = self.list_state.selected() {
            self.utilities[i].selected = !self.utilities[i].selected;
        }
    }

    fn apply_and_quit(&mut self) {
        let results = apply_selections(&self.utilities, &self.install_dir);
        self.results = results.into_iter().map(|(_, msg)| msg).collect();
        self.quit = true;
    }

    fn render(&self, frame: &mut Frame) {
        let chunks = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
            Constraint::Length(5),
        ])
        .split(frame.area());

        let title = Paragraph::new("Utility Installer")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        let items: Vec<ListItem> = self
            .utilities
            .iter()
            .map(|util| {
                let checkbox = if util.selected { "[x]" } else { "[ ]" };

                let status = if util.installed {
                    Span::styled("INSTALLED", Style::default().fg(Color::Green))
                } else {
                    Span::styled("NOT INSTALLED", Style::default().fg(Color::Gray))
                };

                let action = if util.installed && !util.selected {
                    Span::styled(" (uninstall)", Style::default().fg(Color::Yellow))
                } else if !util.installed && util.selected {
                    Span::styled(" (install)", Style::default().fg(Color::Green))
                } else {
                    Span::raw("")
                };

                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{} ", checkbox),
                        Style::default().fg(if util.selected {
                            Color::Green
                        } else if util.initially_installed {
                            Color::Yellow
                        } else {
                            Color::White
                        }),
                    ),
                    Span::styled(
                        format!("{:<20}", util.name),
                        Style::default().fg(Color::White),
                    ),
                    Span::raw(" "),
                    status,
                    action,
                ]))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Utilities"))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, chunks[1], &mut self.list_state.clone());

        let help_text =
            Paragraph::new("↑/↓: Navigate | Space: Select | Enter: Apply | Esc/q: Quit")
                .style(Style::default().fg(Color::DarkGray))
                .block(Block::default().borders(Borders::ALL).title("Controls"));
        frame.render_widget(help_text, chunks[2]);

        if !self.results.is_empty() {
            let msg = self.results.join("\n");
            let message_widget = Paragraph::new(msg.as_str())
                .style(Style::default().fg(Color::White))
                .block(Block::default().borders(Borders::ALL).title("Results"));
            frame.render_widget(message_widget, chunks[3]);
        } else {
            let empty = Paragraph::new("No changes yet")
                .style(Style::default().fg(Color::DarkGray))
                .block(Block::default().borders(Borders::ALL).title("Results"));
            frame.render_widget(empty, chunks[3]);
        }
    }
}

fn find_project_root() -> PathBuf {
    let dirs = [
        std::env::current_dir().ok(),
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf())),
    ];

    for start in dirs.iter().filter_map(|d| d.as_ref()) {
        let mut current = start.clone();
        for _ in 0..10 {
            let install_marker = current.join("install-rs").join("Cargo.toml");
            if install_marker.exists() {
                return current;
            }
            if !current.pop() {
                break;
            }
        }
    }

    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn main() -> io::Result<()> {
    let project_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(find_project_root);

    let install_dir = std::env::var("HOME")
        .map(|home| PathBuf::from(home).join(".local").join("bin"))
        .unwrap_or_else(|_| PathBuf::from("/usr/local/bin"));

    let mut utilities = discover_utilities(&project_dir);

    if utilities.is_empty() {
        eprintln!("No utilities found in {}", project_dir.display());
        std::process::exit(1);
    }

    update_install_status(&mut utilities, &install_dir);

    let mut app = App::new(utilities, install_dir);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| app.render(f))?;

        if app.quit {
            break;
        }

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match (key.modifiers, key.code) {
                (KeyModifiers::NONE, KeyCode::Char('q')) | (KeyModifiers::NONE, KeyCode::Esc) => {
                    app.quit = true;
                }
                (KeyModifiers::NONE, KeyCode::Down) => {
                    app.next();
                }
                (KeyModifiers::NONE, KeyCode::Up) => {
                    app.previous();
                }
                (KeyModifiers::NONE, KeyCode::Char(' ')) => {
                    app.toggle_selection();
                }
                (KeyModifiers::NONE, KeyCode::Enter) => {
                    app.apply_and_quit();
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if !app.results.is_empty() {
        println!();
        for result in &app.results {
            if result.starts_with("Installed:") {
                println!("\x1b[1;32m{}\x1b[0m", result);
            } else if result.starts_with("Uninstalled:") {
                println!("\x1b[1;33m{}\x1b[0m", result);
            } else {
                println!("\x1b[1;31m{}\x1b[0m", result);
            }
        }
        println!();
    }

    Ok(())
}
