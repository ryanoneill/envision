# Claudio to Envision Migration Plan

## Executive Summary

Claudio is a well-architected TUI application for converting EPUB/PDF to audiobooks. It already uses an Elm-inspired TEA pattern, making it highly compatible with Envision. The main problem Envision solves is **enabling Claude to see the TUI** through headless rendering and comprehensive testing infrastructure.

## Current Claudio Architecture

### Strengths (Already TEA-Compatible)
- **State Management**: Centralized `AppState` with `Mode` enum for screens
- **Message Pattern**: `Message` enum for user actions, `Command` for side effects
- **Update Function**: Pure state transitions in `update.rs`
- **View Functions**: Stateless render functions per screen

### Current Testing Infrastructure
- Uses ratatui's `TestBackend` for headless rendering
- Custom `snapshot.rs` module for capturing TUI state
- Snapshot files saved with line numbers and borders
- Tests verify panic-free rendering and content

### Gap Analysis

| Aspect | Claudio Current | Envision Provides |
|--------|-----------------|-------------------|
| Headless Backend | `TestBackend` (basic) | `CaptureBackend` (enhanced cell data, history) |
| App Trait | Custom `App` struct | `App` trait with `Runtime`/`AsyncRuntime` |
| Components | Custom per-screen | Reusable `Component` library |
| Test Harness | Manual assertions | `TestHarness` with fluent API |
| Subscriptions | Manual channel polling | `TickSubscription`, `TimerSubscription` |
| Worker Communication | Custom channels | Can be standardized with patterns |

---

## Migration Plan

### Phase 1: Add Envision Dependency (Non-Breaking)

**Goal**: Use Envision's `CaptureBackend` for enhanced testing without changing app structure.

1. Add `envision` to `Cargo.toml`
2. Replace `TestBackend` with `CaptureBackend` in snapshot.rs
3. Enhance snapshot output with cell-level styling info
4. Update tests to use `TestHarness` for fluent assertions

```rust
// Before (claudio)
let backend = TestBackend::new(120, 40);
let mut terminal = Terminal::new(backend)?;

// After (with Envision)
let backend = CaptureBackend::new(120, 40);
let mut terminal = Terminal::new(backend)?;
// Now has: frame history, ANSI output, JSON export, cell metadata
```

### Phase 2: Adopt Envision Components (Incremental)

Replace custom UI elements with Envision components where applicable:

| Claudio Element | Envision Component | Notes |
|-----------------|-------------------|-------|
| Book list | `SelectableList` | Direct replacement with keyboard nav |
| Progress bar | `ProgressBar` | Add concurrent progress component |
| Status messages | New: `StatusLog` | Needs to be added to Envision |
| Controls bar | New: `KeyHints` | Needs to be added to Envision |
| Screen routing | New: `Screen` | Needs to be added to Envision |

### Phase 3: Full TEA Integration (Optional)

Convert to use Envision's `App` trait and `AsyncRuntime`:

```rust
// Current claudio pattern
impl App {
    async fn run(&mut self) -> Result<PathBuf> {
        loop {
            self.drain_worker_updates();
            self.draw()?;
            self.handle_input()?;
        }
    }
}

// Envision pattern
impl envision::App for Claudio {
    type State = AppState;
    type Message = Message;

    fn update(state: &mut State, msg: Message) -> Command<Message> { ... }
    fn view(state: &State, frame: &mut Frame) { ... }
    fn subscriptions(state: &State) -> Vec<Box<dyn Subscription<Message>>> { ... }
}
```

---

## Components to Add to Envision

Based on claudio's needs and general TUI patterns, these components would benefit the ecosystem:

### 1. StatusLog Component
**Purpose**: Scrolling list of status messages (newest first)

```rust
pub struct StatusLogState {
    messages: VecDeque<StatusMessage>,
    max_messages: usize,
    scroll_position: usize,
}

pub struct StatusMessage {
    text: String,
    level: StatusLevel, // Info, Warning, Error, Success
    timestamp: Option<Instant>,
}

pub enum StatusLogMessage {
    Push(String),
    PushWithLevel(String, StatusLevel),
    Clear,
    ScrollUp,
    ScrollDown,
}
```

**Features**:
- Configurable max messages (default 50)
- Auto-scroll to newest
- Color coding by level
- Timestamp display option

### 2. KeyHints Component
**Purpose**: Display keyboard shortcuts in a status bar

```rust
pub struct KeyHintsState {
    hints: Vec<KeyHint>,
    style: KeyHintsStyle,
}

pub struct KeyHint {
    key: String,      // "Enter", "Esc", "↑/k"
    action: String,   // "Select", "Cancel", "Up"
    enabled: bool,
}

pub enum KeyHintsStyle {
    Inline,    // "Enter Select | Esc Cancel | q Quit"
    Spaced,    // "Enter  Select    Esc  Cancel    q  Quit"
}
```

**Features**:
- Configurable key/action styling
- Enable/disable individual hints
- Responsive layout (hide less important hints when narrow)

### 3. MultiProgress Component
**Purpose**: Display multiple concurrent progress indicators

```rust
pub struct MultiProgressState {
    items: Vec<ProgressItem>,
    max_visible: usize,
}

pub struct ProgressItem {
    id: String,
    label: String,
    current: u64,
    total: u64,
    status: ProgressStatus,
}

pub enum ProgressStatus {
    Pending,
    Active,
    Completed,
    Failed,
}

pub enum MultiProgressMessage {
    Add(ProgressItem),
    Update { id: String, current: u64 },
    Complete(String),
    Remove(String),
}
```

**Features**:
- Concurrent progress bars (like claudio's chapter progress)
- Auto-remove completed items
- Compact single-line per item
- Sort by status or creation order

### 4. Screen/Router Component
**Purpose**: Multi-screen navigation with history

```rust
pub struct ScreenState<S> {
    current: S,
    history: Vec<S>,
    max_history: usize,
}

pub enum ScreenMessage<S> {
    Navigate(S),
    Back,
    Replace(S),
    Clear,
}

pub enum ScreenOutput<S> {
    Changed { from: S, to: S },
    NoHistory,
}
```

**Features**:
- Type-safe screen enum
- Navigation history with back support
- Replace without adding to history
- Transition hooks

### 5. TitleBar Component
**Purpose**: Styled title with optional subtitle and indicators

```rust
pub struct TitleBarState {
    title: String,
    subtitle: Option<String>,
    indicators: Vec<Indicator>,
    alignment: Alignment,
}

pub struct Indicator {
    icon: String,     // "🎵", "⏳", etc.
    text: String,
    style: Style,
}
```

**Features**:
- Configurable alignment
- Multiple indicators (heartbeat, elapsed time, etc.)
- Border styling options

### 6. LoadingList Component
**Purpose**: List with per-item loading states

```rust
pub struct LoadingListState<T> {
    items: Vec<LoadingItem<T>>,
    selected: usize,
}

pub struct LoadingItem<T> {
    data: T,
    loading: bool,
    error: Option<String>,
}

pub enum LoadingListMessage {
    Select(usize),
    SetLoading(usize, bool),
    SetError(usize, Option<String>),
    // ... navigation messages
}
```

**Features**:
- Loading spinner per item
- Error indicator
- Async data loading pattern
- Selection with keyboard navigation

### 7. DurationDisplay Component
**Purpose**: Format and display durations consistently

```rust
pub struct DurationDisplayState {
    duration: Duration,
    format: DurationFormat,
    prefix: Option<String>,
}

pub enum DurationFormat {
    Auto,           // "~8.6 hours" or "~45 minutes"
    HoursMinutes,   // "8h 36m"
    MinutesSeconds, // "45:30"
    Countdown,      // "Time Remaining: 2:30:00"
}
```

**Features**:
- Multiple format options
- Automatic unit selection
- Live countdown support
- Elapsed time tracking

### 8. WorkerBridge Pattern (Not a Component)
**Purpose**: Document pattern for async worker communication

```rust
// Example pattern for integrating with Envision's Command system
pub enum WorkerCommand<M> {
    Spawn { task: BoxFuture<WorkerResult> },
    Cancel { id: Uuid },
}

pub enum WorkerOutput<M> {
    Started { id: Uuid },
    Progress { id: Uuid, progress: f32 },
    Completed { id: Uuid, result: M },
    Failed { id: Uuid, error: String },
}
```

This would be a documented pattern rather than a component, showing how to integrate long-running async tasks with Envision's TEA architecture.

---

## Implementation Priority

### High Priority (Immediate Value)
1. **StatusLog** - Used in almost every TUI app
2. **KeyHints** - Universal need for keyboard-driven UIs
3. **MultiProgress** - Common for batch processing apps

### Medium Priority (Claudio-Specific but Generalizable)
4. **Screen/Router** - Multi-screen apps are common
5. **TitleBar** - Nice-to-have, easy to implement
6. **LoadingList** - Async data loading is common

### Lower Priority (Can Use Existing Patterns)
7. **DurationDisplay** - Can be a utility function
8. **WorkerBridge** - Documentation/example rather than component

---

## Testing Strategy for Claude Visibility

The primary goal is enabling Claude to "see" the TUI. Here's the strategy:

### 1. Enhanced Snapshot System

Replace claudio's custom snapshot with Envision's capabilities:

```rust
// In tests or debug mode
let harness = TestHarness::new(120, 40);
harness.render(|frame| render_app_state(&state, frame));

// For Claude to read
let snapshot = harness.backend().to_string();  // Plain text
let ansi = harness.backend().to_ansi_string(); // With colors
let json = harness.backend().to_json();        // Structured data
```

### 2. Automatic Snapshot on State Change

```rust
impl App for Claudio {
    fn update(state: &mut State, msg: Message) -> Command<Message> {
        let result = handle_message(state, msg);

        #[cfg(feature = "snapshot")]
        save_snapshot_for_claude(state);

        result
    }
}
```

### 3. Integration Test Patterns

```rust
#[test]
fn test_book_selection_workflow() {
    let harness = TestHarness::new(120, 40);
    let mut state = AppState::new(config);

    // Add books
    state.books = vec![book1, book2, book3];

    // Render and verify
    harness.render(|f| render_browse(&state, f));
    harness.assert_contains("Book 1");
    harness.assert_contains("Book 2");

    // Simulate selection
    state.select_next();
    harness.render(|f| render_browse(&state, f));

    // Snapshot for Claude review
    println!("{}", harness.backend());
}
```

---

## Migration Checklist

### Phase 1: Testing Enhancement
- [ ] Add envision dependency
- [ ] Replace TestBackend with CaptureBackend in snapshot.rs
- [ ] Update capture_from_terminal to use CaptureBackend features
- [ ] Add JSON snapshot export option
- [ ] Update tests to use TestHarness where beneficial

### Phase 2: Component Adoption
- [ ] Evaluate SelectableList for book browsing
- [ ] Consider ProgressBar for overall progress
- [ ] Add StatusLog component to Envision
- [ ] Add KeyHints component to Envision
- [ ] Migrate browse screen to use new components

### Phase 3: Full Integration (Optional)
- [ ] Implement App trait for Claudio
- [ ] Use AsyncRuntime for event loop
- [ ] Convert worker updates to Subscription pattern
- [ ] Add Screen component for navigation

---

## Conclusion

Claudio is an excellent candidate for Envision migration because:

1. **Already TEA-compatible** - Minimal architectural changes needed
2. **Complex UI needs** - Will exercise Envision's component library
3. **Real-world validation** - Production app testing the framework
4. **Bidirectional benefit** - Claudio gets better testing, Envision gets new components

The migration can be done incrementally, starting with enhanced testing (Phase 1) and progressively adopting more Envision patterns as beneficial.
