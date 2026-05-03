//! Production App — full lifecycle example for real-world Envision applications.
//!
//! This example demonstrates the complete production lifecycle that every real
//! Envision application needs, going beyond self-contained demos to show:
//!
//! 1. **CLI-style configuration** — Build initial state from external config
//! 2. **`with_args` construction** — Inject the config into `App::init()`
//! 3. **External channel subscription** — Receive progress from a background worker
//! 4. **Lifecycle hooks** — `on_setup_once` / `on_teardown_once` for logging configuration
//! 5. **Background work** — Tokio task simulating file processing with progress
//! 6. **Final state extraction** — Access state after `run_terminal()` returns
//!
//! The application simulates a batch file processor:
//! - Takes a list of "files" to process (configured at startup)
//! - Shows a progress bar and scrolling status log
//! - A background worker sends progress updates via an unbounded channel
//! - Press 'q' to quit early
//! - On exit, prints a summary of how many files were processed
//!
//! Run with: `cargo run --example production_app --features full`

use std::time::Duration;

use envision::app::UnboundedChannelSubscription;
use envision::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph};

// =============================================================================
// Step 1: Configuration (simulates CLI argument parsing)
// =============================================================================

/// Configuration derived from command-line arguments or a config file.
///
/// In a real application, this would come from `clap`, `config`, or similar.
struct AppConfig {
    /// Files to process.
    files: Vec<String>,
    /// Output directory (illustrative; not used in the simulation).
    output_dir: String,
}

impl AppConfig {
    /// Builds a sample configuration as if parsed from CLI args.
    fn from_simulated_args() -> Self {
        Self {
            files: (1..=20).map(|i| format!("document_{:02}.pdf", i)).collect(),
            output_dir: "/tmp/processed".into(),
        }
    }
}

// =============================================================================
// Step 2: Application state (built from config, not from App::init)
// =============================================================================

/// Complete application state for the file processor.
#[derive(Clone)]
struct ProcessorState {
    /// Total number of files to process.
    total_files: usize,
    /// Number of files fully processed so far.
    processed: usize,
    /// The file currently being worked on, if any.
    current_file: Option<String>,
    /// Whether all files have finished processing.
    all_done: bool,

    // Sub-component states
    progress: ProgressBarState,
    log: StatusLogState,
}

impl ProcessorState {
    /// Constructs initial state from an `AppConfig`.
    fn from_config(config: &AppConfig) -> Self {
        let mut progress = ProgressBarState::with_progress(0.0);
        progress.set_label(Some("Processing".to_string()));

        let mut log = StatusLogState::new()
            .with_title("Activity Log")
            .with_max_entries(50);
        log.info(format!(
            "Batch started: {} files queued (output: {})",
            config.files.len(),
            config.output_dir,
        ));

        Self {
            total_files: config.files.len(),
            processed: 0,
            current_file: None,
            all_done: false,
            progress,
            log,
        }
    }
}

// =============================================================================
// Step 3: Messages
// =============================================================================

/// Messages that drive state transitions.
///
/// `FileStarted`, `FileCompleted`, and `AllDone` arrive from the background
/// tokio task via the unbounded channel subscription. `Quit` and `Log` come
/// from keyboard input.
#[derive(Clone, Debug)]
enum ProcessorMsg {
    /// A file has started processing.
    FileStarted(String),
    /// A file has finished processing.
    FileCompleted(String),
    /// All files are done.
    AllDone,
    /// User requested quit.
    Quit,
    /// Delegated status log message (scrolling via Up/Down keys).
    Log(StatusLogMessage),
}

// =============================================================================
// Step 4: App implementation
// =============================================================================

/// The application marker type.
struct ProcessorApp;

impl App for ProcessorApp {
    type State = ProcessorState;
    type Message = ProcessorMsg;
    type Args = AppConfig;

    fn init(config: AppConfig) -> (Self::State, Command<Self::Message>) {
        (ProcessorState::from_config(&config), Command::none())
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Command<Self::Message> {
        match msg {
            ProcessorMsg::FileStarted(name) => {
                state.current_file = Some(name.clone());
                state.log.info(format!("Processing: {}", name));
            }
            ProcessorMsg::FileCompleted(name) => {
                state.processed += 1;
                let fraction = state.processed as f32 / state.total_files as f32;
                state.progress.set_progress(fraction);
                state.log.success(format!("Completed: {}", name));
                state.current_file = None;
            }
            ProcessorMsg::AllDone => {
                state.all_done = true;
                state.progress.set_progress(1.0);
                state.log.success(format!(
                    "All {} files processed! Press 'q' to exit.",
                    state.total_files,
                ));
            }
            ProcessorMsg::Quit => {
                return Command::quit();
            }
            ProcessorMsg::Log(m) => {
                StatusLog::update(&mut state.log, m);
            }
        }
        Command::none()
    }

    fn view(state: &Self::State, frame: &mut Frame) {
        let theme = Theme::catppuccin_mocha();
        let area = frame.area();

        // Main layout: title, progress, current file, log, status bar
        let sections = Layout::vertical([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Progress bar
            Constraint::Length(3), // Current file indicator
            Constraint::Min(6),    // Status log
            Constraint::Length(1), // Bottom status bar
        ])
        .split(area);

        // -- Title --
        let title_text = format!(
            " File Processor  [{}/{}] ",
            state.processed, state.total_files,
        );
        let title = Paragraph::new(title_text)
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            );
        frame.render_widget(title, sections[0]);

        // -- Progress bar --
        let progress_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(
                " Progress ",
                Style::default().add_modifier(Modifier::BOLD),
            ))
            .padding(Padding::horizontal(1));
        let progress_inner = progress_block.inner(sections[1]);
        frame.render_widget(progress_block, sections[1]);
        ProgressBar::view(
            &state.progress,
            &mut RenderContext::new(frame, progress_inner, &theme),
        );

        // -- Current file indicator --
        let current_text = match &state.current_file {
            Some(name) => format!("  Working on: {}", name),
            None if state.all_done => "  Status: All files processed".to_string(),
            None if state.processed == 0 => "  Status: Waiting for worker...".to_string(),
            None => "  Status: Idle (between files)".to_string(),
        };
        let current_style = if state.current_file.is_some() {
            Style::default().fg(Color::Yellow)
        } else if state.all_done {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let current = Paragraph::new(current_text).style(current_style).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        );
        frame.render_widget(current, sections[2]);

        // -- Status log --
        StatusLog::view(
            &state.log,
            &mut RenderContext::new(frame, sections[3], &theme),
        );

        // -- Bottom status bar --
        let status = Line::from(vec![
            Span::styled("[Up/Down] ", Style::default().fg(Color::Cyan)),
            Span::raw("Scroll log  "),
            Span::styled("[q] ", Style::default().fg(Color::Magenta)),
            Span::raw("Quit"),
        ]);
        let status_bar = Paragraph::new(status).alignment(Alignment::Center);
        frame.render_widget(status_bar, sections[4]);
    }

    fn handle_event_with_state(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if let Some(key) = event.as_key() {
            match key.code {
                Key::Char('q') | Key::Esc => {
                    return Some(ProcessorMsg::Quit);
                }
                _ => {}
            }
        }
        // Delegate to the status log for scroll events (Up/Down keys).
        StatusLog::handle_event(&state.log, event, &EventContext::new().focused(true))
            .map(ProcessorMsg::Log)
    }
}

// =============================================================================
// Step 5: Main — full production lifecycle
// =============================================================================

#[tokio::main]
async fn main() -> envision::Result<()> {
    // ── 1. Parse configuration ──────────────────────────────────────────
    let config = AppConfig::from_simulated_args();

    // ── 2. Snapshot the file list before the config is consumed by init ──
    let files = config.files.clone();

    // ── 3. Create the unbounded channel for background worker updates ───
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<ProcessorMsg>();

    // ── 4. Configure runtime with lifecycle hooks ───────────────────────
    //
    // on_setup_once runs *after* the terminal enters raw/alternate-screen mode.
    // In a real app you would redirect stderr to a log file here.
    //
    // on_teardown_once runs *before* the terminal is restored. Flush logs, etc.
    //
    // The `_once` variants accept FnOnce closures, which is the natural choice
    // for one-shot setup/teardown that may capture owned resources.
    let runtime_config = RuntimeConfig::new()
        .on_setup_once(|| {
            // Example: redirect stderr or initialise a tracing subscriber.
            // For this demo we simply do nothing.
            Ok(())
        })
        .on_teardown_once(|| {
            // Example: flush any buffered log output.
            Ok(())
        });

    // ── 5. Create the runtime, passing the config into App::init via with_args.
    let mut runtime = Runtime::<ProcessorApp, _>::terminal_builder()?
        .with_args(config)
        .config(runtime_config)
        .build()?;

    // ── 6. Subscribe to background worker updates ───────────────────────
    //
    // Messages sent to `tx` will appear as `ProcessorMsg` in the runtime's
    // event loop, exactly as if they were produced by `handle_event` or
    // `on_tick`.
    runtime.subscribe(UnboundedChannelSubscription::new(rx));

    // ── 7. Spawn the background worker ──────────────────────────────────
    //
    // This simulates a CPU-bound or I/O-bound task running off the main
    // thread, sending progress updates through the channel.
    tokio::spawn(async move {
        for file in &files {
            // Notify the UI that we are starting this file.
            if tx.send(ProcessorMsg::FileStarted(file.clone())).is_err() {
                return; // Runtime shut down; stop the worker.
            }

            // Simulate processing time.
            tokio::time::sleep(Duration::from_millis(400)).await;

            // Notify the UI that this file is done.
            if tx.send(ProcessorMsg::FileCompleted(file.clone())).is_err() {
                return;
            }
        }

        // Signal completion.
        let _ = tx.send(ProcessorMsg::AllDone);
    });

    // ── 8. Run the interactive event loop ───────────────────────────────
    //
    // `run_terminal()` blocks until the user presses 'q' or all files
    // finish and the user quits. It returns ownership of the final state.
    let final_state = runtime.run_terminal().await?;

    // ── 9. Use the final state after the TUI has exited ─────────────────
    //
    // The terminal has been restored to normal mode at this point, so
    // regular println! works.
    println!();
    println!("Processing summary");
    println!("──────────────────");
    println!(
        "Files processed: {}/{}",
        final_state.processed, final_state.total_files,
    );
    if final_state.processed < final_state.total_files {
        println!(
            "Cancelled early ({} files remaining)",
            final_state.total_files - final_state.processed,
        );
    } else {
        println!("All files processed successfully.");
    }

    Ok(())
}
