#!/usr/bin/env rust-script

//! ```cargo
//! [dependencies]
//! crossterm = "0.25.0"
//! chrono = "0.4.38"
//! chrono-tz = "0.9.0"
//! crossterm = "0.25.0"
//! dateparser = "0.2.1"
//! serde = { version = "1.0.204", features = ["derive"] }
//! serde_json = "1.0.120"
//! strip-ansi-escapes = "0.2.0"
//! terminal-menu = "3.0.0"
//! typst = "0.11.1"
//! ```

extern crate chrono;
extern crate chrono_tz;
extern crate dateparser;
extern crate terminal_menu;
extern crate typst;

use chrono::{offset::Local, TimeZone};

use std::{
    collections::HashMap,
    env::consts::OS,
    fs,
    io::Write,
    num::ParseIntError,
    process::Command,
    str::FromStr,
    sync::{Arc, RwLock},
};

use crossterm::style::{Color, Colored};
use serde::Deserialize;
use terminal_menu::{
    back_button, button, label, menu, mut_menu, run, scroll, string, submenu, TerminalMenuItem,
    TerminalMenuStruct,
};

pub struct MenuBuilder {
    items: Vec<TerminalMenuItem>,
}

impl MenuBuilder {
    pub fn add_item(mut self, item: TerminalMenuItem) -> Self {
        self.items.push(item);
        self
    }
    pub fn add_button<T: Into<String>>(self, name: T) -> Self {
        self.add_item(button(name))
    }
    pub fn add_back_button<T: Into<String>>(self, name: T) -> Self {
        self.add_item(back_button(name))
    }
    pub fn add_label<T: Into<String>>(self, text: T) -> Self {
        self.add_item(label(text))
    }
    pub fn add_scroll<T: Into<String>, T2: IntoIterator>(self, name: T, values: T2) -> Self
    where
        T2::Item: Into<String>,
    {
        self.add_item(scroll(name, values))
    }
    pub fn add_string<T: Into<String>, T2: Into<String>>(
        self,
        name: T,
        default: T2,
        allow_empty: bool,
    ) -> Self {
        self.add_item(string(name, default, allow_empty))
    }
    pub fn add_menu<T: Into<String> + Clone>(self, name: T, sub_menu_builder: MenuBuilder) -> Self {
        self.add_item(submenu(name, sub_menu_builder.items))
    }
    pub fn colorize_prev(mut self, color: Color) -> Self {
        self.items
            .pop()
            .and_then(|item: TerminalMenuItem| Some(self.items.push(item.colorize(color))));
        self
    }
    pub fn build(self: MenuBuilder) -> Arc<RwLock<TerminalMenuStruct>> {
        menu(self.items)
    }
}

fn menu_builder() -> MenuBuilder {
    MenuBuilder { items: vec![] }
}

#[derive(Deserialize, Debug)]
struct NotebookinatorEntryTypeMetadata {
    pub data: (Vec<ThemeMetadata>,),
}

#[derive(Deserialize, Debug)]
struct ThemeMetadata(pub String, Option<Vec<EntryTypeMetadata>>);

#[derive(Deserialize, Debug)]
struct EntryTypeMetadata(String, EntryTypeMetadataValue);

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum EntryTypeMetadataValue {
    ColorString(String),
    ColorObject(EntryTypeMetadataObject),
}

#[derive(Deserialize, Debug)]
struct EntryTypeMetadataObject {
    pub color: String,
}
#[derive(Debug, Clone)]
struct EntryType {
    name: String,
    color: Color,
}

pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

impl EntryType {
    pub fn from_string_pair((name, color_str): (String, String)) -> Self {
        let hex_color = color_str
            .trim_start_matches("rgb(\"#")
            .trim_end_matches("\")");
        let hex_bytes = decode_hex(hex_color).expect("Failed to parse rgb color string");
        if hex_bytes.len() != 3 {
            panic!("rbg color string was not of length 3")
        }
        EntryType {
            name: name,
            color: Color::Rgb {
                r: *hex_bytes.get(0).unwrap(),
                g: *hex_bytes.get(1).unwrap(),
                b: *hex_bytes.get(2).unwrap(),
            },
        }
    }

    pub fn from_string_pairs(
        iter: Box<dyn Iterator<Item = (String, String)>>,
    ) -> Box<dyn Iterator<Item = Self>> {
        Box::new(iter.map(|pair| Self::from_string_pair(pair)))
    }
}

fn query_entry_type_metadata() -> Box<dyn Iterator<Item = EntryType>> {
    let raw_metadata = String::from_utf8(
        Command::new(if OS == "windows" {
            "C:\\Program Files\\Git\\usr\\bin\\bash.exe"
        } else {
            "bash"
        })
        .arg("-c")
        .arg(
            "typst query - '<entry-types>' --field value <<EOF
#import \"@local/notebookinator:1.0.1\": themes
#metadata(
  dictionary(themes).pairs().map(((name, theme)) => {
    let entry-metadata = dictionary(theme.components).pairs().find((
      (key, _value),
    ) => key == \"entry-type-metadata\")
    if (entry-metadata == none) {
      return (name, entry-metadata)
    }
    return (name, entry-metadata.at(1).pairs())
  }),
) <entry-types>
EOF",
        )
        .output()
        .expect("failed to query typst for entry-type-metadata")
        .stdout,
    )
    .unwrap();
    let wrapped_metadata = format!("{{ \"data\": {} }}", raw_metadata);
    let deserialized_metadata: NotebookinatorEntryTypeMetadata =
        serde_json::de::from_str(&wrapped_metadata).expect("failed to parse metadata");
    let theme_entries_map: HashMap<String, Vec<(String, String)>> = deserialized_metadata
        .data
        .0
        .into_iter()
        .filter_map(
            |theme: ThemeMetadata| -> Option<(String, Vec<(String, String)>)> {
                let theme_name = theme.0;
                let entry_types = theme.1;
                return entry_types.and_then(|entry_types: Vec<EntryTypeMetadata>| {
                    Some((
                        theme_name,
                        entry_types
                            .into_iter()
                            .map(|entry_type| -> (String, String) {
                                let entry_name = entry_type.0;
                                let color: String = match entry_type.1 {
                                    EntryTypeMetadataValue::ColorString(str) => str,
                                    EntryTypeMetadataValue::ColorObject(
                                        EntryTypeMetadataObject { color },
                                    ) => color,
                                };
                                (entry_name, color)
                            })
                            .collect(),
                    ))
                });
            },
        )
        .collect();
    use typst::syntax::{
        ast::{
            Arg::Named,
            AstNode,
            Expr::{FieldAccess, FuncCall, Show},
            Markup,
        },
        parse,
    };

    // attempt to get the theme from ./main.typ
    let contents = fs::read_to_string("./main.typ").expect("Failed to read ./main.typ");
    let untyped_ast = parse(contents.as_str());
    let ast = Markup::from_untyped(&untyped_ast).expect("Failed to parse ./main.typ's AST");
    let themes = ast
        .exprs()
        .filter_map(|expr| match expr {
            Show(show_rule) => Some(show_rule.transform()),
            _ => None,
        })
        .filter_map(|expr| match expr {
            FuncCall(func_call) => Some(func_call),
            _ => None,
        })
        .filter(|func| match func.callee() {
            FieldAccess(field_access) => field_access.target().to_untyped().text() == "notebook",
            _ => false,
        })
        .map(|func| {
            func.args()
                .items()
                .into_iter()
                .filter_map(|arg| match arg {
                    Named(named_arg) => Some(named_arg),
                    _ => None,
                })
                .filter(|arg| arg.name().as_str() == "theme")
                .map(|arg| arg.expr().to_untyped().to_owned().into_text())
        })
        .flatten();
    for user_theme in themes {
        for (theme, entries) in theme_entries_map.iter() {
            if user_theme.contains(theme) {
                return EntryType::from_string_pairs(Box::new(entries.to_owned().into_iter()));
            }
        }
    }
    let default_theme = theme_entries_map
        .get_key_value("radial")
        .or_else(|| theme_entries_map.iter().next())
        .expect("Failed to find any themes with entry types in notebookinator");
    eprintln!(
        "Could not find theme in ./main.typ, defaulting to {}.",
        default_theme.0
    );
    return EntryType::from_string_pairs(Box::new(default_theme.1.to_owned().into_iter()));
}

fn make_date_time_str(date: chrono::DateTime<Local>) -> String {
    date.format("datetime(year: %Y, month: %m, day: %d)")
        .to_string()
}

fn main() -> Result<(), String> {
    let entry_types = query_entry_type_metadata();
    let entry_types_vec: Vec<EntryType> = entry_types.collect();
    let todays_date = chrono::Local::now();
    let todays_date_str = todays_date.format("%F").to_string();
    // let mut entry_type_sub_menu = menu_builder();
    // for entry_type in query_entry_type_metadata()
    let my_menu = menu_builder()
        .add_label("-----------------")
        .add_label("Make a new entry!")
        .add_label("-----------------")
        .add_scroll("section", vec!["body", "frontmatter", "appendix"])
        .add_string("title", "", false)
        .add_scroll(
            "type",
            entry_types_vec.iter().map(|e| {
                format!("\x1B[{}m", Colored::ForegroundColor(e.color).to_string()) + e.name.as_str()
            }),
        )
        .add_string("date", todays_date_str, false)
        .add_string(
            "author",
            Command::new("git")
                .arg("config")
                .arg("--get")
                .arg("user.name")
                .output()
                .and_then(|output| {
                    Ok(String::from(
                        String::from_utf8(output.stdout)
                            .unwrap_or(String::new())
                            .trim(),
                    ))
                })
                .unwrap_or(String::new()),
            false,
        )
        .add_string("witness", "", true)
        .add_button("enter!")
        .colorize_prev(Color::Green)
        .build();

    run(&my_menu);
    let my_mut_menu = mut_menu(&my_menu);

    let date = dateparser::parse_with_timezone(my_mut_menu.selection_value("date"), &Local)
        .ok()
        .and_then(|date| Local.from_local_datetime(&date.naive_local()).earliest())
        .or_else(|| {
            eprintln!("failed to parse date!");
            None
        })
        .unwrap_or(todays_date);
    let date_string = make_date_time_str(date);
    let date_str = date_string.as_str();
    let title_input = my_mut_menu.selection_value("title");
    let title = title_input.split('/').last().unwrap();
    let section = my_mut_menu.selection_value("section");
    let entry_type_string = String::from_utf8(strip_ansi_escapes::strip(
        my_mut_menu.selection_value("type"),
    ))
    .unwrap();
    let entry_type = entry_type_string.as_str();
    let author = my_mut_menu.selection_value("author");
    let witness = my_mut_menu.selection_value("witness");

    if title.len() == 0 {
        return Err(String::from_str("title must be specified!").unwrap());
    };

    let entry_content = format!(
        "#import \"/packages.typ\": *
#import components: *
// TODO: add comment
#show: create-entry.with(
    section: \"{section}\",
    title: \"{title}\",
    type: \"{entry_type}\",
    date: {date_str},
    author: \"{author}\",
    witness: \"{witness}\",
)"
    );

    let entry_dir_path = "./entries/".to_owned()
        + (title_input
            .to_lowercase()
            .replace(" ", "_")
            .trim_end_matches("/"));
    let entry_file_path_vec: Vec<&str> = entry_dir_path.split("/").collect();
    let entry_file_name = entry_file_path_vec.last().unwrap();
    {
        let mut new_dir_path = String::from_str(entry_file_path_vec.first().unwrap()).unwrap();
        for path_part in entry_file_path_vec.iter().skip(1) {
            new_dir_path += "/";
            new_dir_path += path_part;

            fs::create_dir(new_dir_path.clone())
                .or_else(|err| {
                    if err.kind() == std::io::ErrorKind::AlreadyExists {
                        Ok(())
                    } else {
                        Err(err)
                    }
                })
                .expect(
                    format!("Failed to make part of entry directory: ({})", new_dir_path).as_str(),
                );
        }
    }

    let entry_file_path = &format!("{}/{}.typ", entry_dir_path, entry_file_name);
    let mut entry_file = fs::File::create_new(entry_file_path)
        .expect(format!("Failed to make entry typst file ({})", entry_file_path).as_str());
    entry_file
        .write_all(entry_content.as_bytes())
        .expect("Failed to write to entry typst file");
    entry_file
        .flush()
        .expect("Failed to flush to entry typst file");

    let mut entries_file = fs::File::options()
        .append(true)
        .open("./entries/entries.typ")
        .expect("Failed to open ./entries/entries.typ");
    entries_file
        .write_all(
            format!(
                "\n\n#include \"{}\"",
                entry_file_path.trim_start_matches(".")
            )
            .as_bytes(),
        )
        .expect("Failed to write to ./entries/entries.typ");
    entries_file
        .flush()
        .expect("Failed to flush to ./entries/entries.typ");

    Ok(())
}
