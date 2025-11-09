# ratatui-cfg

A crate that lets you automatically create a settings
menu based on any serializable/deserializable struct

## Usage Examples

### Basic Menu

```rust
use ratatui_cfg::{ConfigMenuTrait, MenuController, render_menu};
use ratatui_cfg_derive::ConfigMenu;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use serde::{Serialize, Deserialize};
use std::io;

#[derive(Debug, Clone, Serialize, Deserialize, ConfigMenu)]
struct Config {
    volume: u32,
    fullscreen: bool,
    max_fps: u32,
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let config = Config {
        volume: 80,
        fullscreen: false,
        max_fps: 60,
    };

    let mut controller = MenuController::new(config);
    let result = run(&mut terminal, &mut controller);
    ratatui::restore();
    result
}

fn run(
    terminal: &mut ratatui::DefaultTerminal,
    controller: &mut MenuController,
) -> io::Result {
    loop {
        terminal.draw(|frame| {
            render_menu(frame, controller, frame.area());
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Up => controller.menu_state.previous(),
                KeyCode::Down => controller.menu_state.next(),
                KeyCode::Enter => {
                    if controller.is_current_boolean() {
                        controller.toggle_boolean()?;
                    } else if controller.is_current_submenu() {
                        controller.enter_submenu()?;
                    } else {
                        controller.start_editing();
                    }
                }
                KeyCode::Esc => {
                    if controller.editing_mode {
                        controller.cancel_editing();
                    } else if controller.menu_state.can_go_back() {
                        controller.menu_state.go_back();
                    }
                }
                KeyCode::Char('s') => {
                    controller.save_to_file("config.toml")?;
                }
                KeyCode::Char('r') => {
                    *controller = MenuController::load_from_file("config.toml")?;
                }
                KeyCode::Char(c) if controller.editing_mode => {
                    controller.handle_edit_input(c);
                }
                KeyCode::Backspace if controller.editing_mode => {
                    controller.handle_backspace();
                }
                KeyCode::Delete if controller.editing_mode => {
                    controller.handle_delete();
                }
                KeyCode::Left if controller.editing_mode => {
                    controller.move_cursor_left();
                }
                KeyCode::Right if controller.editing_mode => {
                    controller.move_cursor_right();
                }
                _ => {}
            }
        }
    }

    Ok(())
}
```

## Key Bindings

The default key bindings in the menu system are:

| Key     | Action                                      |
| ------- | ------------------------------------------- |
| Up/Down | Navigate menu items                         |
| Enter   | Toggle boolean / Edit field / Enter submenu |
| Esc     | Cancel editing / Go back to parent menu     |
| s       | Save configuration to file                  |
| r       | Reload configuration from file              |
| q       | Quit application                            |

During text editing:

- Left/Right: Move cursor
- Backspace/Delete: Delete characters
- Enter: Save changes
- Esc: Cancel editing

## Rendering

The menu UI consists of four sections:

1. **Breadcrumb**: Shows navigation path (e.g., "Config > Database > Connection")
2. **Settings List**: Displays all fields with values and indicators (> for submenus, [] for vectors)
3. **Status**: Shows current mode (editing, ready) and edit buffer
4. **Help**: Conteaxt-sensitive keyboard shortcuts

## Type Support

Supported field types:

- Primitives: `bool`, `i8`-`i128`, `u8`-`u128`, `f32`, `f64`, `String`, `usize`, `isize`
- Wrappers: `Option<T>`, `Vec<T>`
- Custom: Any type implementing `ConfigMenuTrait`

## Requirements

Your configuration types must implement:

- `Debug`
- `Clone`
- `Serialize` (serde)
- `Deserialize` (serde)
- `ConfigMenuTrait` (via `#[derive(ConfigMenu)]`)
