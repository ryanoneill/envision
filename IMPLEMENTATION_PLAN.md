# Next Features Implementation Plan

## Analysis of Current State

### Implemented Components (10 total)

**Input Components:**
- Button - Clickable button with keyboard activation
- Checkbox - Toggleable boolean input
- RadioGroup - Mutually exclusive option selection
- InputField - Single-line text input
- TextArea - Multi-line text editing

**Display Components:**
- ProgressBar - Visual progress indicator
- Spinner - Loading indicator with multiple styles
- SelectableList - Scrollable list with keyboard navigation

**Navigation Components:**
- Tabs - Horizontal tab navigation

**Utility Components:**
- FocusManager - Focus coordination

### Gap Analysis

The framework has strong fundamentals but is missing several essential TUI components:

**High Priority Missing Components:**
1. **Modal/Dialog** - Popup overlays for confirmations, alerts, forms
2. **Select/Dropdown** - Compact dropdown selection (different from SelectableList)
3. **Menu** - Menu bar for application commands
4. **Table** - Data grid with columns and rows
5. **StatusBar** - Bottom status bar showing app state

**Medium Priority:**
6. **Tree** - Hierarchical data display
7. **Split/Pane** - Resizable layouts
8. **Scrollbar** - Explicit scrollbar widget
9. **Toast/Notification** - Temporary messages

## Proposed Implementation: Phase 1

For this PR, I propose implementing **three complementary components** that work together to enable rich application UIs:

### 1. Modal Component

**Purpose:** Display popup dialogs over the main content with an overlay backdrop.

**Features:**
- Semi-transparent overlay that dims background
- Centered or positioned dialog box
- Focus trapping (Tab cycles within modal)
- Esc to close (optional)
- Configurable title and content area
- Supports custom borders and styling

**State:**
```rust
pub struct ModalState {
    pub visible: bool,
    pub title: String,
    pub content_height: u16,
    pub content_width: u16,
    pub can_close: bool,
}
```

**Messages:**
- `Show` - Display the modal
- `Hide` - Hide the modal
- `Toggle` - Toggle visibility

**Output:**
- `Closed` - User requested to close modal

**Use Cases:**
- Confirmation dialogs ("Are you sure?")
- Alert messages
- Forms and data entry
- Help screens

### 2. Select Component

**Purpose:** Dropdown selection menu (more compact than SelectableList for forms).

**Features:**
- Compact closed state showing selected value
- Opens dropdown list on activation
- Keyboard navigation (Up/Down arrows, Enter to select)
- Type-ahead filtering (optional)
- Configurable option list
- Placeholder text when nothing selected

**State:**
```rust
pub struct SelectState {
    pub options: Vec<String>,
    pub selected_index: Option<usize>,
    pub is_open: bool,
    pub focused: bool,
    pub placeholder: String,
}
```

**Messages:**
- `Toggle` - Open/close dropdown
- `Open` - Open dropdown
- `Close` - Close dropdown
- `SelectNext` - Move selection down
- `SelectPrevious` - Move selection up
- `Confirm` - Confirm current selection
- `Focus` - Receive focus
- `Blur` - Lose focus

**Output:**
- `Changed(Option<usize>)` - Selection changed
- `Submitted(usize)` - User confirmed selection

**Use Cases:**
- Form inputs with predefined options
- Settings/preferences selection
- Command palettes
- Filter controls

### 3. Menu Component

**Purpose:** Horizontal menu bar for application commands.

**Features:**
- Horizontal list of menu items
- Keyboard navigation (Left/Right arrows)
- Activation with Enter or mouse
- Highlighting of selected item
- Support for disabled items
- Dividers/separators

**State:**
```rust
pub struct MenuState {
    pub items: Vec<MenuItem>,
    pub selected_index: usize,
    pub focused: bool,
}

pub struct MenuItem {
    pub label: String,
    pub enabled: bool,
    pub is_divider: bool,
}
```

**Messages:**
- `SelectNext` - Move to next menu item
- `SelectPrevious` - Move to previous menu item
- `Activate` - Activate selected item
- `Focus` - Receive focus
- `Blur` - Lose focus

**Output:**
- `ItemActivated(usize)` - Menu item was activated

**Use Cases:**
- Application menu bar (File, Edit, View, etc.)
- Context menus
- Action bars
- Navigation bars

## Implementation Strategy

### Development Approach

For each component, follow the established pattern:

1. **Create component file** in `src/component/`
2. **Define State, Message, Output** types
3. **Implement Component trait** (init, update, view)
4. **Implement Focusable trait** (if applicable)
5. **Add comprehensive tests** (20-40 tests)
6. **Add rustdoc with examples**
7. **Update lib.rs exports**
8. **Create example** in `examples/`

### Testing Strategy

Each component will include tests for:
- Initialization and default state
- Message handling
- Focus management
- Edge cases
- View rendering (using CaptureBackend)
- Integration scenarios

### Documentation Strategy

Each component will have:
- Module-level documentation with overview
- Type-level documentation for State/Message/Output
- Function-level documentation for all public APIs
- Usage examples in rustdoc
- Standalone example program

## Success Criteria

✅ All three components implemented following established patterns
✅ Comprehensive test coverage (>95%)
✅ Full rustdoc documentation with examples
✅ At least one example program demonstrating usage
✅ All tests pass (`cargo test`)
✅ No clippy warnings (`cargo clippy`)
✅ Code formatted (`cargo fmt`)

## Future Phases

**Phase 2 (Next PR):**
- Table component (data grids)
- StatusBar component
- Toast/Notification component

**Phase 3 (Future PR):**
- Tree component (hierarchical data)
- Split/Pane layouts
- DatePicker component

## Timeline

This implementation will be completed in a single PR:
- Modal: ~400-500 lines with tests
- Select: ~450-550 lines with tests
- Menu: ~350-450 lines with tests
- Examples: ~150-200 lines total
- **Total estimated:** ~1,350-1,700 lines

This aligns with the established component complexity (300-850 lines per component including tests).
