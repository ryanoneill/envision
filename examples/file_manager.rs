#![allow(
    clippy::collapsible_if,
    clippy::single_match,
    clippy::type_complexity,
    clippy::collapsible_match
)]
// =============================================================================
// File Manager — Reference Application #4
// =============================================================================
//
// A file manager demonstrating the navigation and compound component suite:
//
// - **FileBrowser**: Directory listing with sort, filter, hidden files
// - **Tree**: Hierarchical directory tree sidebar
// - **CodeBlock**: File preview with syntax highlighting
// - **DiffViewer**: Side-by-side diff view
// - **SplitPanel**: Resizable left/right panes
// - **Breadcrumb**: Path navigation
// - **StatusBar**: File info and key hints
// - **CommandPalette**: Quick actions
//
// Run: cargo run --example file_manager --features full
//
// Layout:
// ┌ /home/user/project ─────────────────────────────────────────┐
// │ src > components > button.rs                                 │
// ├──────────────┬──────────────────────────────────────────────┤
// │ ▸ src/       │ fn main() {                                   │
// │   ▸ comp/    │     println!("Hello");                        │
// │   mod.rs     │ }                                             │
// │ Cargo.toml   │                                               │
// │ README.md    │                                               │
// ├──────────────┴──────────────────────────────────────────────┤
// │ button.rs │ 1.2 KB │ Rust │ F1: help │ /: filter            │
// └─────────────────────────────────────────────────────────────┘

use envision::prelude::*;

// Types not re-exported via the prelude
use envision::component::code_block::highlight::Language;
use envision::component::file_browser::{FileEntry, FileSortField};

// ---------------------------------------------------------------------------
// Focus
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum Focus {
    Browser,
    Preview,
}

// ---------------------------------------------------------------------------
// Preview mode
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
enum PreviewMode {
    Code,
    Diff,
}

// ---------------------------------------------------------------------------
// App message
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
enum Msg {
    Browser(FileBrowserMessage),
    Code(CodeBlockMessage),
    Diff(DiffViewerMessage),
    Split(SplitPanelMessage),
    Crumb(BreadcrumbMessage),
    Palette(CommandPaletteMessage),

    FocusToggle,
    TogglePalette,
    TogglePreviewMode,
    Quit,
}

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct State {
    browser: FileBrowserState,
    code: CodeBlockState,
    diff: DiffViewerState,
    split: SplitPanelState,
    breadcrumb: BreadcrumbState,
    palette: CommandPaletteState,
    status: StatusBarState,
    focus: FocusManager<Focus>,
    preview_mode: PreviewMode,
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

struct FileManager;

impl App for FileManager {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let palette_items = vec![
            PaletteItem::new("toggle-hidden", "Toggle Hidden Files").with_shortcut("Ctrl+H"),
            PaletteItem::new("toggle-diff", "Toggle Diff View").with_shortcut("Ctrl+D"),
            PaletteItem::new("sort-name", "Sort by Name"),
            PaletteItem::new("sort-size", "Sort by Size"),
            PaletteItem::new("sort-ext", "Sort by Extension"),
            PaletteItem::new("quit", "Quit").with_shortcut("Ctrl+Q"),
        ];

        let entries = sample_entries("/home/user/project");

        let mut status = StatusBarState::new();
        update_status_items(&mut status, &entries[0]);

        let state = State {
            browser: FileBrowserState::new("/home/user/project", entries),
            code: sample_code_block("main.rs"),
            diff: sample_diff(),
            split: SplitPanelState::new(SplitOrientation::Vertical).with_ratio(0.3),
            breadcrumb: BreadcrumbState::from_path("/home/user/project", "/"),
            palette: CommandPaletteState::new(palette_items)
                .with_title("File Manager")
                .with_placeholder("Type a command..."),
            status,
            focus: FocusManager::with_initial_focus(vec![Focus::Browser, Focus::Preview]),
            preview_mode: PreviewMode::Code,
        };

        (state, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Browser(m) => {
                if let Some(output) = FileBrowser::update(&mut state.browser, m) {
                    match output {
                        FileBrowserOutput::FileSelected(entry) => {
                            // Update code preview for selected file
                            let name = entry.name().to_string();
                            state.code = sample_code_block(&name);
                            update_status_items(&mut state.status, &entry);
                        }
                        FileBrowserOutput::DirectoryEntered(path) => {
                            state.breadcrumb = BreadcrumbState::from_path(&path, "/");
                        }
                        FileBrowserOutput::NavigatedBack(path) => {
                            state.breadcrumb = BreadcrumbState::from_path(&path, "/");
                        }
                        _ => {}
                    }
                }
            }
            Msg::Code(m) => {
                CodeBlock::update(&mut state.code, m);
            }
            Msg::Diff(m) => {
                DiffViewer::update(&mut state.diff, m);
            }
            Msg::Split(m) => {
                SplitPanel::update(&mut state.split, m);
            }
            Msg::Crumb(m) => {
                if let Some(output) = Breadcrumb::update(&mut state.breadcrumb, m) {
                    match output {
                        BreadcrumbOutput::Selected(idx) => {
                            // Navigate to breadcrumb segment
                            let path = rebuild_path(&state.breadcrumb, idx);
                            let entries = sample_entries(&path);
                            state.browser = FileBrowserState::new(&path, entries);
                            state.breadcrumb = BreadcrumbState::from_path(&path, "/");
                        }
                        _ => {}
                    }
                }
            }
            Msg::Palette(m) => {
                if let Some(output) = CommandPalette::update(&mut state.palette, m) {
                    match output {
                        CommandPaletteOutput::Selected(item) => match item.id.as_str() {
                            "toggle-hidden" => {
                                let show = !state.browser.show_hidden();
                                state.browser.set_show_hidden(show);
                            }
                            "toggle-diff" => {
                                return Self::update(state, Msg::TogglePreviewMode);
                            }
                            "sort-name" => {
                                FileBrowser::update(
                                    &mut state.browser,
                                    FileBrowserMessage::SetSort(FileSortField::Name),
                                );
                            }
                            "sort-size" => {
                                FileBrowser::update(
                                    &mut state.browser,
                                    FileBrowserMessage::SetSort(FileSortField::Size),
                                );
                            }
                            "sort-ext" => {
                                FileBrowser::update(
                                    &mut state.browser,
                                    FileBrowserMessage::SetSort(FileSortField::Extension),
                                );
                            }
                            "quit" => return Command::quit(),
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }

            Msg::FocusToggle => {
                state.focus.focus_next();
            }
            Msg::TogglePalette => {
                if state.palette.is_visible() {
                    state.palette.dismiss();
                } else {
                    state.palette.show();
                }
            }
            Msg::TogglePreviewMode => {
                state.preview_mode = match state.preview_mode {
                    PreviewMode::Code => PreviewMode::Diff,
                    PreviewMode::Diff => PreviewMode::Code,
                };
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = Theme::default();
        let area = frame.area();

        // Layout: breadcrumb + split panel + status bar
        let chunks = Layout::vertical([
            Constraint::Length(1), // Breadcrumb
            Constraint::Min(0),    // Split panel (browser + preview)
            Constraint::Length(1), // Status bar
        ])
        .split(area);

        let browser_focused = state.focus.is_focused(&Focus::Browser);
        let preview_focused = state.focus.is_focused(&Focus::Preview);

        // Breadcrumb
        Breadcrumb::view(
            &state.breadcrumb,
            &mut RenderContext::new(frame, chunks[0], &theme).focused(browser_focused),
        );

        // Split panel: browser (left) + preview (right)
        let (left, right) = state.split.layout(chunks[1]);

        FileBrowser::view(
            &state.browser,
            &mut RenderContext::new(frame, left, &theme).focused(browser_focused),
        );

        match state.preview_mode {
            PreviewMode::Code => {
                CodeBlock::view(
                    &state.code,
                    &mut RenderContext::new(frame, right, &theme).focused(preview_focused),
                );
            }
            PreviewMode::Diff => {
                DiffViewer::view(
                    &state.diff,
                    &mut RenderContext::new(frame, right, &theme).focused(preview_focused),
                );
            }
        }

        // Status bar
        StatusBar::view(
            &state.status,
            &mut RenderContext::new(frame, chunks[2], &theme),
        );

        // Command palette overlay
        if state.palette.is_visible() {
            CommandPalette::view(
                &state.palette,
                &mut RenderContext::new(frame, area, &theme).focused(true),
            );
        }
    }

    fn handle_event(_event: &Event) -> Option<Msg> {
        None
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        let key = event.as_key()?;
        let ctrl = key.modifiers.ctrl();

        // Command palette gets priority
        if state.palette.is_visible() {
            return CommandPalette::handle_event(&state.palette, event, &EventContext::default())
                .map(Msg::Palette);
        }

        // Global shortcuts
        match key.code {
            Key::Char('q') if ctrl => return Some(Msg::Quit),
            Key::Char('p') if ctrl => return Some(Msg::TogglePalette),
            Key::Char('h') if ctrl => {
                return Some(Msg::Palette(CommandPaletteMessage::Confirm));
            }
            Key::Char('d') if ctrl => return Some(Msg::TogglePreviewMode),
            Key::Tab => return Some(Msg::FocusToggle),
            Key::Esc => return Some(Msg::Quit),
            _ => {}
        }

        // Browser-focused
        if state.focus.is_focused(&Focus::Browser) {
            return FileBrowser::handle_event(&state.browser, event, &EventContext::default())
                .map(Msg::Browser);
        }

        // Preview-focused
        if state.focus.is_focused(&Focus::Preview) {
            return match state.preview_mode {
                PreviewMode::Code => {
                    CodeBlock::handle_event(&state.code, event, &EventContext::default())
                        .map(Msg::Code)
                }
                PreviewMode::Diff => {
                    DiffViewer::handle_event(&state.diff, event, &EventContext::default())
                        .map(Msg::Diff)
                }
            };
        }

        // Fall through: split panel resize and breadcrumb navigation
        if let Some(m) = SplitPanel::handle_event(&state.split, event, &EventContext::default()) {
            return Some(Msg::Split(m));
        }
        Breadcrumb::handle_event(&state.breadcrumb, event, &EventContext::default()).map(Msg::Crumb)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn update_status_items(status: &mut StatusBarState, entry: &FileEntry) {
    let size_str = match entry.size() {
        Some(s) if s >= 1024 => format!("{:.1} KB", s as f64 / 1024.0),
        Some(s) => format!("{} B", s),
        None => "---".to_string(),
    };
    let kind = if entry.is_dir() { "Dir" } else { "File" };

    status.set_left(vec![StatusBarItem::new(format!(
        "{} | {} | {}",
        entry.name(),
        size_str,
        kind
    ))]);
    status.set_right(vec![StatusBarItem::new(
        "Tab: focus | Ctrl+P: commands | Ctrl+D: diff",
    )]);
}

fn rebuild_path(breadcrumb: &BreadcrumbState, up_to_idx: usize) -> String {
    let segments = breadcrumb.segments();
    let path: String = segments
        .iter()
        .take(up_to_idx + 1)
        .map(|s| s.label().to_string())
        .collect::<Vec<_>>()
        .join("/");
    format!("/{}", path)
}

fn sample_entries(path: &str) -> Vec<FileEntry> {
    match path {
        "/home/user/project" => vec![
            FileEntry::directory("src", "/home/user/project/src"),
            FileEntry::directory("tests", "/home/user/project/tests"),
            FileEntry::directory("examples", "/home/user/project/examples"),
            FileEntry::file("Cargo.toml", "/home/user/project/Cargo.toml").with_size(1250),
            FileEntry::file("Cargo.lock", "/home/user/project/Cargo.lock").with_size(48200),
            FileEntry::file("README.md", "/home/user/project/README.md").with_size(3400),
            FileEntry::file("main.rs", "/home/user/project/main.rs").with_size(890),
            FileEntry::file(".gitignore", "/home/user/project/.gitignore").with_size(120),
            FileEntry::file(".env", "/home/user/project/.env").with_size(64),
        ],
        "/home/user/project/src" => vec![
            FileEntry::directory("components", "/home/user/project/src/components"),
            FileEntry::file("lib.rs", "/home/user/project/src/lib.rs").with_size(2100),
            FileEntry::file("app.rs", "/home/user/project/src/app.rs").with_size(4500),
            FileEntry::file("theme.rs", "/home/user/project/src/theme.rs").with_size(1800),
        ],
        _ => vec![
            FileEntry::file("mod.rs", format!("{}/mod.rs", path)).with_size(640),
            FileEntry::file("tests.rs", format!("{}/tests.rs", path)).with_size(2300),
        ],
    }
}

fn sample_code_block(filename: &str) -> CodeBlockState {
    let (code, lang) = match filename {
        "main.rs" | "lib.rs" | "app.rs" | "mod.rs" => (
            "use std::io;\n\
             \n\
             fn main() -> io::Result<()> {\n\
             \x20   let config = Config::load()?;\n\
             \x20   let app = App::new(config);\n\
             \x20   app.run()\n\
             }",
            Language::Rust,
        ),
        "Cargo.toml" => (
            "[package]\n\
             name = \"my-project\"\n\
             version = \"0.1.0\"\n\
             edition = \"2021\"\n\
             \n\
             [dependencies]\n\
             envision = \"0.10\"",
            Language::Toml,
        ),
        "README.md" => (
            "# My Project\n\
             \n\
             A sample project demonstrating envision components.\n\
             \n\
             ## Usage\n\
             \n\
             ```bash\n\
             cargo run\n\
             ```",
            Language::Plain,
        ),
        "theme.rs" => (
            "use ratatui::style::{Color, Style};\n\
             \n\
             pub struct Theme {\n\
             \x20   pub fg: Color,\n\
             \x20   pub bg: Color,\n\
             \x20   pub accent: Color,\n\
             }\n\
             \n\
             impl Default for Theme {\n\
             \x20   fn default() -> Self {\n\
             \x20       Self {\n\
             \x20           fg: Color::White,\n\
             \x20           bg: Color::Black,\n\
             \x20           accent: Color::Cyan,\n\
             \x20       }\n\
             \x20   }\n\
             }",
            Language::Rust,
        ),
        _ => (
            "// File contents not available\n\
             // Select a file to preview",
            Language::Plain,
        ),
    };

    CodeBlockState::new()
        .with_code(code)
        .with_language(lang)
        .with_title(filename)
        .with_line_numbers(true)
}

fn sample_diff() -> DiffViewerState {
    let old = "\
fn main() {
    println!(\"Hello\");
}";
    let new = "\
fn main() {
    let name = \"World\";
    println!(\"Hello, {}!\", name);
}";

    DiffViewerState::from_texts(old, new)
        .with_title("main.rs")
        .with_old_label("main.rs (old)")
        .with_new_label("main.rs (new)")
        .with_line_numbers(true)
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() -> envision::Result<()> {
    let mut vt = Runtime::<FileManager, _>::virtual_builder(100, 30).build()?;

    // Simulate selecting a file using vt.dispatch() for proper command processing
    vt.dispatch(Msg::Browser(FileBrowserMessage::Down));
    vt.dispatch(Msg::Browser(FileBrowserMessage::Down));
    vt.dispatch(Msg::Browser(FileBrowserMessage::Down));
    vt.tick()?;

    println!("File Manager — Reference Application");
    println!("=====================================");
    println!();
    println!("{}", vt.display());
    println!();
    println!("This demonstrates:");
    println!("  - FileBrowser with sorting and filtering");
    println!("  - CodeBlock with syntax highlighting and line numbers");
    println!("  - DiffViewer for file comparisons");
    println!("  - SplitPanel with resizable panes");
    println!("  - Breadcrumb path navigation");
    println!("  - CommandPalette for quick actions");
    println!("  - FocusManager + EventContext focus routing");
    println!("  - StatusBar with file info");

    Ok(())
}
