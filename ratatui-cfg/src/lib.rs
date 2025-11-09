#![allow(clippy::only_used_in_recursion)]

pub use ratatui_cfg_derive::ConfigMenu;

use {
    color_eyre::eyre::{Error, Result},
    ratatui::{
        Frame,
        layout::{Constraint, Direction, Layout, Rect},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    },
    serde::{Deserialize, Serialize},
    std::{any::Any, fmt::Debug, path::Path},
    undo::{Edit, Record},
};

#[derive(Clone, Debug, PartialEq)]
pub enum FieldType {
    String,
    Bool,
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
    F32,
    F64,
    Nested,
    Unknown,
}

type Getter = Box<dyn Fn(&dyn Any) -> Option<String>>;
type Setter = Box<dyn Fn(&mut dyn Any, String) -> Result<(), String>>;
type NestedGetter = Box<dyn Fn(&dyn Any) -> Option<Box<dyn Any>>>;
type NestedMetadataGetter = Box<dyn Fn() -> Vec<FieldMetadata>>;
type NestedSetter = Box<dyn Fn(&mut dyn Any, Box<dyn Any>) -> Result<(), String>>;

pub struct FieldMetadata {
    pub name: &'static str,
    pub is_nested: bool,
    pub is_option: bool,
    pub is_vec: bool,
    pub field_type: FieldType,
    pub getter: Getter,
    pub setter: Setter,
    pub nested_getter: Option<NestedGetter>,
    pub nested_metadata_getter: Option<NestedMetadataGetter>,
    pub nested_setter: Option<NestedSetter>,
}

pub trait ConfigMenuTrait: Debug + Clone + Serialize + for<'de> Deserialize<'de> + 'static {
    fn get_field_metadata() -> Vec<FieldMetadata>;
    fn get_menu_title() -> &'static str;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub fn format_field_value<T: Debug>(value: &T) -> String {
    format!("{:?}", value)
}

fn strip_debug_quotes(s: &str) -> String {
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        let inner = &s[1..s.len() - 1];
        inner.replace(r#"\""#, "\"").replace(r"\\", r"\")
    } else {
        s.to_string()
    }
}

pub trait ParsableField: Sized {
    fn parse_from_string(value: String) -> Result<Self, String>;
}

impl ParsableField for String {
    fn parse_from_string(value: String) -> Result<Self, String> {
        Ok(value)
    }
}

impl ParsableField for bool {
    fn parse_from_string(value: String) -> Result<Self, String> {
        value
            .parse()
            .map_err(|_| format!("Failed to parse '{}'", value))
    }
}

impl ParsableField for i8 {
    fn parse_from_string(value: String) -> Result<Self, String> {
        value
            .parse()
            .map_err(|_| format!("Failed to parse '{}'", value))
    }
}

impl ParsableField for i16 {
    fn parse_from_string(value: String) -> Result<Self, String> {
        value
            .parse()
            .map_err(|_| format!("Failed to parse '{}'", value))
    }
}

impl ParsableField for i32 {
    fn parse_from_string(value: String) -> Result<Self, String> {
        value
            .parse()
            .map_err(|_| format!("Failed to parse '{}'", value))
    }
}

impl ParsableField for i64 {
    fn parse_from_string(value: String) -> Result<Self, String> {
        value
            .parse()
            .map_err(|_| format!("Failed to parse '{}'", value))
    }
}

impl ParsableField for i128 {
    fn parse_from_string(value: String) -> Result<Self, String> {
        value
            .parse()
            .map_err(|_| format!("Failed to parse '{}'", value))
    }
}

impl ParsableField for isize {
    fn parse_from_string(value: String) -> Result<Self, String> {
        value
            .parse()
            .map_err(|_| format!("Failed to parse '{}'", value))
    }
}

impl ParsableField for u8 {
    fn parse_from_string(value: String) -> Result<Self, String> {
        value
            .parse()
            .map_err(|_| format!("Failed to parse '{}'", value))
    }
}

impl ParsableField for u16 {
    fn parse_from_string(value: String) -> Result<Self, String> {
        value
            .parse()
            .map_err(|_| format!("Failed to parse '{}'", value))
    }
}

impl ParsableField for u32 {
    fn parse_from_string(value: String) -> Result<Self, String> {
        value
            .parse()
            .map_err(|_| format!("Failed to parse '{}'", value))
    }
}

impl ParsableField for u64 {
    fn parse_from_string(value: String) -> Result<Self, String> {
        value
            .parse()
            .map_err(|_| format!("Failed to parse '{}'", value))
    }
}

impl ParsableField for u128 {
    fn parse_from_string(value: String) -> Result<Self, String> {
        value
            .parse()
            .map_err(|_| format!("Failed to parse '{}'", value))
    }
}

impl ParsableField for usize {
    fn parse_from_string(value: String) -> Result<Self, String> {
        value
            .parse()
            .map_err(|_| format!("Failed to parse '{}'", value))
    }
}

impl ParsableField for f32 {
    fn parse_from_string(value: String) -> Result<Self, String> {
        value
            .parse()
            .map_err(|_| format!("Failed to parse '{}'", value))
    }
}

impl ParsableField for f64 {
    fn parse_from_string(value: String) -> Result<Self, String> {
        value
            .parse()
            .map_err(|_| format!("Failed to parse '{}'", value))
    }
}

impl<T> ParsableField for T
where
    T: ConfigMenuTrait,
{
    fn parse_from_string(value: String) -> Result<Self, String> {
        toml::from_str(&value).map_err(|e| format!("Failed to parse nested config: {}", e))
    }
}

pub fn parse_and_set<T>(field: &mut T, value: String) -> Result<(), String>
where
    T: ParsableField,
{
    *field = T::parse_from_string(value)?;
    Ok(())
}

#[derive(Clone)]
pub struct ConfigEdit {
    _field_path: Vec<String>,
    old_value: String,
    new_value: String,
}

impl Edit for ConfigEdit {
    type Target = String;
    type Output = ();

    fn edit(&mut self, target: &mut String) {
        *target = self.new_value.clone();
    }

    fn undo(&mut self, target: &mut String) {
        *target = self.old_value.clone();
    }
}

pub struct MenuController<T: ConfigMenuTrait> {
    pub config: T,
    pub menu_state: MenuState,
    pub history: Record<ConfigEdit>,
    pub editing_mode: bool,
    pub edit_buffer: String,
    pub edit_cursor: usize,
}

impl<T: ConfigMenuTrait> MenuController<T> {
    pub fn new(config: T) -> Self {
        let menu_state = MenuState::new(&config);
        Self {
            config,
            menu_state,
            history: Record::new(),
            editing_mode: false,
            edit_buffer: String::new(),
            edit_cursor: 0,
        }
    }

    pub fn start_editing(&mut self) {
        if let Some(item) = self.menu_state.get_current_item()
            && !item.is_submenu
            && !item.is_vec_container
        {
            self.editing_mode = true;

            if item.field_type == FieldType::String {
                self.edit_buffer = strip_debug_quotes(&item.value);
            } else {
                self.edit_buffer = item.value.clone();
            }

            self.edit_cursor = self.edit_buffer.len();
        }
    }

    pub fn toggle_boolean(&mut self) -> Result<(), String> {
        if let Some(item) = self.menu_state.get_current_item()
            && item.field_type == FieldType::Bool
            && !item.is_submenu
            && !item.is_vec_container
        {
            let new_value = if item.value == "true" {
                "false"
            } else {
                "true"
            };

            let field_path = self.menu_state.get_current_field_path();
            let result = self.apply_edit_at_path(&field_path, new_value);

            if result.is_ok() {
                let current_path = self.menu_state.get_navigation_path();
                self.menu_state = MenuState::new(&self.config);

                for field_name in current_path {
                    if let Err(e) = self
                        .menu_state
                        .enter_submenu_by_name(&self.config, &field_name)
                    {
                        eprintln!("Failed to restore navigation: {}", e);
                        break;
                    }
                }
            }

            result
        } else {
            Ok(())
        }
    }

    pub fn finish_editing(&mut self) -> Result<(), String> {
        if !self.editing_mode {
            return Ok(());
        }

        let new_value = self.edit_buffer.clone();
        let field_path = self.menu_state.get_current_field_path();

        let result = self.apply_edit_at_path(&field_path, &new_value);

        if result.is_ok() {
            let current_path = self.menu_state.get_navigation_path();
            self.menu_state = MenuState::new(&self.config);

            for field_name in current_path {
                if let Err(e) = self
                    .menu_state
                    .enter_submenu_by_name(&self.config, &field_name)
                {
                    eprintln!("Failed to restore navigation: {}", e);
                    break;
                }
            }
        }

        self.editing_mode = false;
        result
    }

    fn apply_edit_at_path(&mut self, field_path: &[String], new_value: &str) -> Result<(), String> {
        if field_path.is_empty() {
            return Err("Empty field path".to_string());
        }

        if field_path.len() == 1 {
            Self::set_field_on_config(&mut self.config, &field_path[0], new_value)
        } else {
            self.set_nested_field_recursive(field_path, new_value)
        }
    }

    fn set_field_on_config<U: ConfigMenuTrait>(
        config: &mut U,
        field_name: &str,
        value: &str,
    ) -> Result<(), String> {
        let metadata = U::get_field_metadata();
        let field_meta = metadata
            .iter()
            .find(|m| m.name == field_name)
            .ok_or_else(|| format!("Field '{}' not found", field_name))?;

        (field_meta.setter)(config.as_any_mut(), value.to_string())
    }

    fn set_nested_field_recursive(
        &mut self,
        field_path: &[String],
        new_value: &str,
    ) -> Result<(), String> {
        let root_field = &field_path[0];
        let metadata = T::get_field_metadata();

        let field_meta = metadata
            .iter()
            .find(|m| m.name == root_field)
            .ok_or_else(|| format!("Field '{}' not found", root_field))?;

        if !field_meta.is_nested {
            return Err(format!("Field '{}' is not nested", root_field));
        }

        let nested_getter = field_meta
            .nested_getter
            .as_ref()
            .ok_or_else(|| "No nested getter available".to_string())?;

        let nested_any = (nested_getter)(self.config.as_any())
            .ok_or_else(|| format!("Failed to get nested field '{}'", root_field))?;

        let updated_nested = self.update_nested_any(
            nested_any,
            &field_path[1..],
            new_value,
            field_meta.nested_metadata_getter.as_ref(),
        )?;

        let nested_setter = field_meta
            .nested_setter
            .as_ref()
            .ok_or_else(|| "No nested setter available".to_string())?;

        (nested_setter)(self.config.as_any_mut(), updated_nested)
    }

    fn update_nested_any(
        &self,
        mut nested_any: Box<dyn Any>,
        remaining_path: &[String],
        new_value: &str,
        metadata_getter: Option<&NestedMetadataGetter>,
    ) -> Result<Box<dyn Any>, String> {
        if remaining_path.is_empty() {
            return Ok(nested_any);
        }

        let metadata =
            metadata_getter.ok_or_else(|| "No metadata getter for nested field".to_string())?();

        let field_name = &remaining_path[0];
        let field_meta = metadata
            .iter()
            .find(|m| m.name == field_name)
            .ok_or_else(|| format!("Field '{}' not found in nested structure", field_name))?;

        if remaining_path.len() == 1 {
            (field_meta.setter)(nested_any.as_mut(), new_value.to_string())?;
            Ok(nested_any)
        } else {
            if !field_meta.is_nested {
                return Err(format!("Field '{}' is not nested", field_name));
            }

            let inner_nested_getter = field_meta
                .nested_getter
                .as_ref()
                .ok_or_else(|| "No nested getter for inner field".to_string())?;

            let inner_nested = (inner_nested_getter)(nested_any.as_ref())
                .ok_or_else(|| format!("Failed to get nested field '{}'", field_name))?;

            let updated_inner = self.update_nested_any(
                inner_nested,
                &remaining_path[1..],
                new_value,
                field_meta.nested_metadata_getter.as_ref(),
            )?;

            let inner_setter = field_meta
                .nested_setter
                .as_ref()
                .ok_or_else(|| "No nested setter for inner field".to_string())?;

            (inner_setter)(nested_any.as_mut(), updated_inner)?;
            Ok(nested_any)
        }
    }

    pub fn enter_submenu(&mut self) -> Result<(), String> {
        let item = self
            .menu_state
            .get_current_item()
            .ok_or_else(|| "No item selected".to_string())?;

        if !item.is_submenu {
            return Err("Current item is not a submenu".to_string());
        }

        let field_name = item.label.clone();
        self.menu_state
            .enter_submenu_by_name(&self.config, &field_name)
    }

    pub fn cancel_editing(&mut self) {
        self.editing_mode = false;
        self.edit_buffer.clear();
        self.edit_cursor = 0;
    }

    pub fn is_current_submenu(&self) -> bool {
        self.menu_state
            .get_current_item()
            .is_some_and(|item| item.is_submenu)
    }

    pub fn is_current_boolean(&self) -> bool {
        self.menu_state
            .get_current_item()
            .is_some_and(|item| item.field_type == FieldType::Bool && !item.is_submenu)
    }

    pub fn handle_edit_input(&mut self, c: char) {
        self.edit_buffer.insert(self.edit_cursor, c);
        self.edit_cursor += 1;
    }

    pub fn handle_backspace(&mut self) {
        if self.edit_cursor > 0 {
            self.edit_buffer.remove(self.edit_cursor - 1);
            self.edit_cursor -= 1;
        }
    }

    pub fn handle_delete(&mut self) {
        if self.edit_cursor < self.edit_buffer.len() {
            self.edit_buffer.remove(self.edit_cursor);
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.edit_cursor > 0 {
            self.edit_cursor -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.edit_cursor < self.edit_buffer.len() {
            self.edit_cursor += 1;
        }
    }

    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        let toml_string = toml::to_string_pretty(&self.config)?;
        std::fs::write(path, toml_string)?;
        Ok(())
    }

    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
        let contents = std::fs::read_to_string(path)?;
        let config: T = toml::from_str(&contents)?;
        Ok(Self::new(config))
    }
}

pub struct MenuState {
    pub current_selection: usize,
    pub items: Vec<MenuItem>,
    pub list_state: ListState,
    pub breadcrumb: Vec<String>,
    pub menu_stack: Vec<MenuLevel>,
}

pub struct MenuLevel {
    pub items: Vec<MenuItem>,
    pub selection: usize,
    pub title: String,
    pub field_path: Vec<String>,
}

#[derive(Clone)]
pub struct MenuItem {
    pub label: String,
    pub value: String,
    pub is_submenu: bool,
    pub is_vec_container: bool,
    pub field_type: FieldType,
}

impl MenuState {
    pub fn new<T: ConfigMenuTrait>(config: &T) -> Self {
        let metadata = T::get_field_metadata();
        let items = Self::build_menu_items(config, &metadata);

        let mut list_state = ListState::default();
        if !items.is_empty() {
            list_state.select(Some(0));
        }

        Self {
            current_selection: 0,
            items: items.clone(),
            list_state,
            breadcrumb: vec![T::get_menu_title().to_string()],
            menu_stack: vec![MenuLevel {
                items,
                selection: 0,
                title: T::get_menu_title().to_string(),
                field_path: vec![],
            }],
        }
    }

    fn build_menu_items<T: ConfigMenuTrait>(
        config: &T,
        metadata: &[FieldMetadata],
    ) -> Vec<MenuItem> {
        metadata
            .iter()
            .map(|field| {
                let value = (field.getter)(config.as_any()).unwrap_or_else(|| "N/A".to_string());

                let value_display = if field.is_option {
                    if value.contains("None") {
                        "<not set>".to_string()
                    } else {
                        value.replace("Some(", "").replace(")", "")
                    }
                } else {
                    value
                };

                MenuItem {
                    label: field.name.to_string(),
                    value: value_display,
                    is_submenu: field.is_nested,
                    is_vec_container: field.is_vec,
                    field_type: field.field_type.clone(),
                }
            })
            .collect()
    }

    pub fn enter_submenu_by_name<T: ConfigMenuTrait>(
        &mut self,
        parent_config: &T,
        field_name: &str,
    ) -> Result<(), String> {
        let metadata = T::get_field_metadata();
        let field_meta = metadata
            .iter()
            .find(|m| m.name == field_name)
            .ok_or_else(|| format!("Field '{}' not found", field_name))?;

        if !field_meta.is_nested {
            return Err(format!("Field '{}' is not a nested structure", field_name));
        }

        let nested_getter = field_meta
            .nested_getter
            .as_ref()
            .ok_or_else(|| format!("No nested getter for field '{}'", field_name))?;

        let nested_any = (nested_getter)(parent_config.as_any())
            .ok_or_else(|| format!("Cannot access nested configuration for '{}'", field_name))?;

        let nested_metadata_getter = field_meta
            .nested_metadata_getter
            .as_ref()
            .ok_or_else(|| format!("No nested metadata getter for field '{}'", field_name))?;

        let nested_metadata = (nested_metadata_getter)();

        let nested_items = Self::build_menu_items_from_any(&*nested_any, &nested_metadata);

        let current_level = self.menu_stack.last().unwrap();
        let mut new_field_path = current_level.field_path.clone();
        new_field_path.push(field_name.to_string());

        let new_level = MenuLevel {
            items: nested_items.clone(),
            selection: 0,
            title: field_name.to_string(),
            field_path: new_field_path,
        };

        self.menu_stack.push(new_level);
        self.breadcrumb.push(field_name.to_string());
        self.items = nested_items;
        self.current_selection = 0;
        self.list_state.select(Some(0));

        Ok(())
    }

    fn build_menu_items_from_any(
        nested_any: &dyn Any,
        metadata: &[FieldMetadata],
    ) -> Vec<MenuItem> {
        metadata
            .iter()
            .map(|field| {
                let value = (field.getter)(nested_any).unwrap_or_else(|| "N/A".to_string());

                let value_display = if field.is_option {
                    if value.contains("None") {
                        "<not set>".to_string()
                    } else {
                        value.replace("Some(", "").replace(")", "")
                    }
                } else {
                    value
                };

                MenuItem {
                    label: field.name.to_string(),
                    value: value_display,
                    is_submenu: field.is_nested,
                    is_vec_container: field.is_vec,
                    field_type: field.field_type.clone(),
                }
            })
            .collect()
    }

    pub fn get_current_field_path(&self) -> Vec<String> {
        let mut path = self
            .menu_stack
            .last()
            .map(|level| level.field_path.clone())
            .unwrap_or_default();

        if let Some(item) = self.get_current_item() {
            path.push(item.label.clone());
        }

        path
    }

    pub fn get_navigation_path(&self) -> Vec<String> {
        self.menu_stack
            .iter()
            .skip(1)
            .map(|level| level.title.clone())
            .collect()
    }

    pub fn next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => (i + 1) % self.items.len(),
            None => 0,
        };
        self.list_state.select(Some(i));
        self.current_selection = i;
    }

    pub fn previous(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.current_selection = i;
    }

    pub fn get_current_item(&self) -> Option<&MenuItem> {
        self.items.get(self.current_selection)
    }

    pub fn can_go_back(&self) -> bool {
        self.menu_stack.len() > 1
    }

    pub fn go_back(&mut self) {
        if self.can_go_back() {
            self.menu_stack.pop();
            self.breadcrumb.pop();

            if let Some(prev_level) = self.menu_stack.last() {
                self.items = prev_level.items.clone();
                self.current_selection = prev_level.selection;
                self.list_state.select(Some(self.current_selection));
            }
        }
    }
}

pub fn render_menu<T: ConfigMenuTrait>(
    frame: &mut Frame,
    controller: &mut MenuController<T>,
    area: Rect,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(area);

    let breadcrumb = controller.menu_state.breadcrumb.join(" > ");
    let breadcrumb_widget = Paragraph::new(breadcrumb)
        .block(Block::default().borders(Borders::ALL).title("Navigation"))
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(breadcrumb_widget, chunks[0]);

    let items: Vec<ListItem> = controller
        .menu_state
        .items
        .iter()
        .map(|item| {
            let indicator = if item.is_submenu {
                " >"
            } else if item.is_vec_container {
                " []"
            } else {
                ""
            };
            let content = format!("{}: {}{}", item.label, item.value, indicator);
            ListItem::new(Line::from(vec![Span::styled(
                content,
                Style::default().fg(Color::White),
            )]))
        })
        .collect();

    let items_widget = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Settings"))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    frame.render_stateful_widget(
        items_widget,
        chunks[1],
        &mut controller.menu_state.list_state,
    );

    let status_text = if controller.editing_mode {
        format!("Editing: {}", controller.edit_buffer)
    } else {
        "Ready".to_string()
    };

    let status_widget = Paragraph::new(status_text)
        .block(Block::default().borders(Borders::ALL).title("Status"))
        .style(if controller.editing_mode {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::Gray)
        });
    frame.render_widget(status_widget, chunks[2]);

    if controller.editing_mode {
        frame.set_cursor_position((
            chunks[2].x + controller.edit_cursor as u16 + 10,
            chunks[2].y + 1,
        ));
    }

    let help_text = if controller.editing_mode {
        "Esc: Cancel | Enter: Save | Left/Right: Move cursor | Backspace/Del: Delete"
    } else if controller.is_current_submenu() {
        "Up/Down: Navigate | Enter: Open submenu | Esc: Back | s: Save | q: Quit"
    } else if controller.is_current_boolean() {
        "Up/Down: Navigate | Enter: Toggle | Esc: Back | s: Save | r: Reload | q: Quit"
    } else if controller.menu_state.can_go_back() {
        "Up/Down: Navigate | Enter: Edit | Esc: Back | s: Save | r: Reload | q: Quit"
    } else {
        "Up/Down: Navigate | Enter: Edit | s: Save | r: Reload | q: Quit"
    };

    let help_widget = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .style(Style::default().fg(Color::Gray));
    frame.render_widget(help_widget, chunks[3]);
}
