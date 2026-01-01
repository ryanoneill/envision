# Select and Menu Components Implementation

## Summary

This PR adds two new UI components to the envision framework:

1. **Select** - A dropdown selection component for choosing from a list of options
2. **Menu** - A horizontal menu bar for application commands

## Components Added

### Select Component

A compact dropdown menu for selecting a single option from a list.

**Features:**
- Displays selected value when closed, all options when opened
- Keyboard navigation (Up/Down to navigate, Enter to confirm)
- Placeholder text when nothing is selected
- Disabled state support
- Implements `Focusable` trait

**Types:**
- `Select` - The component struct
- `SelectState` - Component state (options, selected_index, is_open, etc.)
- `SelectMessage` - Messages (Open, Close, Toggle, SelectNext, SelectPrevious, Confirm)
- `SelectOutput` - Outputs (Changed, Submitted)

**Usage:**
```rust
use envision::component::{Select, SelectMessage, SelectOutput, SelectState, Component};

let mut state = SelectState::new(vec!["Small", "Medium", "Large"]);

// Open dropdown and select
Select::update(&mut state, SelectMessage::Open);
Select::update(&mut state, SelectMessage::SelectNext);
let output = Select::update(&mut state, SelectMessage::Confirm);
```

### Menu Component

A horizontal menu bar for application commands and navigation.

**Features:**
- Horizontal layout of menu items
- Keyboard navigation (Left/Right arrows)
- Item activation with Enter
- Disabled item support
- Implements `Focusable` trait

**Types:**
- `Menu` - The component struct
- `MenuState` - Component state (items, selected_index, focused)
- `MenuItem` - Individual menu item (label, enabled)
- `MenuMessage` - Messages (SelectNext, SelectPrevious, Activate, SelectItem)
- `MenuOutput` - Outputs (ItemActivated)

**Usage:**
```rust
use envision::component::{Menu, MenuMessage, MenuOutput, MenuState, MenuItem, Component};

let mut state = MenuState::new(vec![
    MenuItem::new("File"),
    MenuItem::new("Edit"),
    MenuItem::disabled("View"),
]);

// Navigate and activate
Menu::update(&mut state, MenuMessage::SelectNext);
let output = Menu::update(&mut state, MenuMessage::Activate);
```

## Design Decisions

### Modal vs Dialog

The original plan included a Modal component, but this was omitted because:
- The existing `Dialog` component already provides modal overlay functionality
- `Dialog` is more full-featured with button support and structured actions
- Adding Modal would create redundancy with no clear differentiation

### Component Patterns

Both components follow the established patterns:
- TEA (The Elm Architecture) with State/Message/Output
- `Component` trait implementation
- `Focusable` trait for keyboard focus management
- Comprehensive test coverage
- Full rustdoc documentation

## Files Changed

| File | Action |
|------|--------|
| `src/component/select.rs` | Created - Select component (~730 lines) |
| `src/component/menu.rs` | Created - Menu component (~650 lines) |
| `src/component/mod.rs` | Modified - Added module declarations and exports |
| `src/lib.rs` | Modified - Added re-exports |
| `IMPLEMENTATION_PLAN.md` | Created - This document |

## Test Coverage

- **Select**: 28 unit tests covering state management, navigation, selection, disabled state, and rendering
- **Menu**: 24 unit tests covering item management, navigation, activation, and rendering

## Future Components

Remaining components that could be added:
- Tree (hierarchical data display)
- Accordion (collapsible sections)
- Breadcrumb (navigation trail)
- Tooltip (hover hints)
- StatusBar (bottom status indicators)
