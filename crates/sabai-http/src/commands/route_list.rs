use std::env;
use std::fs::File;
use std::io::{self, IsTerminal};
use std::process::{Command as Process, Stdio};
use std::sync::Arc;

use sabai_core::{App, Command, Method, Result};

use crate::Router;
use crate::router::RouteRow;

const BLUE: &str = "34";
const BLUE_BOLD: &str = "1;34";
const GRAY: &str = "90";
const RED: &str = "31";
const WHITE: &str = "37";
const YELLOW: &str = "33";

// Laravel's characters, written as escapes so the source stays ASCII.
const NAME_SEPARATOR: &str = " \u{203a} ";
const ELLIPSIS: char = '\u{2026}';

/// `route:list`: every route with its name and handler, like `php artisan route:list`.
pub struct RouteListCommand {
    rows: Vec<RouteRow>,
}

impl RouteListCommand {
    /// A `route:list` command for `router`'s routes.
    pub fn new(router: &Router) -> Self {
        Self {
            rows: router.describe(),
        }
    }
}

impl Command for RouteListCommand {
    fn name(&self) -> &str {
        "route:list"
    }

    fn description(&self) -> &str {
        "List all registered routes"
    }

    fn handle(&self, _app: &Arc<App>, _args: &[String]) -> Result<()> {
        let paint = Paint(io::stdout().is_terminal() && env::var_os("NO_COLOR").is_none());
        for line in table(&self.rows, terminal_width(), paint) {
            println!("{line}");
        }
        Ok(())
    }
}

/// ANSI colors, only when printing to a terminal.
#[derive(Clone, Copy)]
struct Paint(bool);

impl Paint {
    fn color(self, code: &str, text: &str) -> String {
        match self.0 {
            true => format!("\x1b[{code}m{text}\x1b[0m"),
            false => text.to_owned(),
        }
    }
}

struct Line {
    method: String,
    uri: String,
    action: String,
}

impl Line {
    fn new(row: &RouteRow) -> Self {
        let method = match row.method {
            Method::GET => "GET|HEAD".to_owned(),
            ref method => method.to_string(),
        };
        let uri = match row.path.trim_matches('/') {
            "" => "/".to_owned(),
            uri => uri.to_owned(),
        };
        let action = match (&row.name, short_action(row.action)) {
            (Some(name), Some(action)) => format!("{name}{NAME_SEPARATOR}{action}"),
            (Some(name), None) => name.clone(),
            (None, Some(action)) => action,
            (None, None) => String::new(),
        };
        Self {
            method,
            uri,
            action,
        }
    }

    // Laravel's layout: the method column is the widest method plus six, and dots fill the rest.
    fn render(&self, widest_method: usize, width: usize, paint: Paint) -> String {
        let spaces = " ".repeat(widest_method + 6 - self.method.len());
        let used = chars(&self.method) + spaces.len() + chars(&self.uri);
        let dots = self.dots(used, width);
        let action = self.fitted_action(used + chars(&dots), width);
        let line = format!(
            "  {} {spaces}{}{}",
            paint_method(&self.method, paint),
            paint_uri(&self.uri, paint),
            paint.color(GRAY, &dots)
        );
        match action.is_empty() {
            true => line,
            false => format!("{line} {}", paint.color(GRAY, &action)),
        }
    }

    fn dots(&self, used: usize, width: usize) -> String {
        let gap = usize::from(!self.action.is_empty());
        match width.saturating_sub(used + chars(&self.action) + 6 + gap) {
            0 => String::new(),
            count => format!(" {}", ".".repeat(count)),
        }
    }

    // Like Laravel, a line wider than the terminal loses the end of its action, never its path.
    fn fitted_action(&self, used: usize, width: usize) -> String {
        if self.action.is_empty() || used + chars(&self.action) <= width.saturating_sub(6) {
            return self.action.clone();
        }
        let room = width.saturating_sub(used + 7);
        self.action.chars().take(room).chain([ELLIPSIS]).collect()
    }
}

fn table(rows: &[RouteRow], width: usize, paint: Paint) -> Vec<String> {
    let lines = sorted_by_uri(rows);
    let widest_method = lines
        .iter()
        .map(|line| line.method.len())
        .max()
        .unwrap_or(0);
    let count = format!("Showing [{}] routes", lines.len());
    let indent = " ".repeat(width.saturating_sub(count.len() + 2));

    let mut output = vec![String::new()];
    for line in &lines {
        output.push(line.render(widest_method, width, paint));
    }
    output.push(String::new());
    output.push(format!("{indent}{}", paint.color(BLUE_BOLD, &count)));
    output.push(String::new());
    output
}

// Inserting after equal paths keeps routes on one path in the order added, like Laravel's stable
// `sortBy`, without compiling a whole sort algorithm for a list of a few hundred routes.
fn sorted_by_uri(rows: &[RouteRow]) -> Vec<Line> {
    let mut lines: Vec<Line> = Vec::with_capacity(rows.len());
    for row in rows {
        let line = Line::new(row);
        let position = lines.partition_point(|sorted| sorted.uri <= line.uri);
        lines.insert(position, line);
    }
    lines
}

/// `PostController@index` for a controller method, the path inside the app for a function,
/// and nothing for a closure, which Laravel shows by its name alone.
fn short_action(type_name: &str) -> Option<String> {
    if type_name.contains("{{closure}}") {
        return None;
    }
    let segments: Vec<&str> = type_name.split("::").collect();
    match segments.as_slice() {
        [.., owner, method] if owner.starts_with(char::is_uppercase) => {
            Some(format!("{owner}@{method}"))
        }
        [_app, path @ ..] if !path.is_empty() => Some(path.join("::")),
        _ => Some(type_name.to_owned()),
    }
}

fn paint_method(method: &str, paint: Paint) -> String {
    let mut painted = String::new();
    for (index, verb) in method.split('|').enumerate() {
        if index > 0 {
            painted.push_str(&paint.color(GRAY, "|"));
        }
        painted.push_str(&paint.color(verb_color(verb), verb));
    }
    painted
}

fn verb_color(verb: &str) -> &'static str {
    match verb {
        "GET" => BLUE,
        "POST" | "PUT" | "PATCH" => YELLOW,
        "DELETE" => RED,
        _ => GRAY,
    }
}

fn paint_uri(uri: &str, paint: Paint) -> String {
    let mut painted = String::new();
    let mut rest = uri;
    while let (Some(start), Some(end)) = (rest.find('{'), rest.find('}')) {
        painted.push_str(&paint.color(WHITE, &rest[..start]));
        painted.push_str(&paint.color(YELLOW, &rest[start..=end]));
        rest = &rest[end + 1..];
    }
    painted.push_str(&paint.color(WHITE, rest));
    painted
}

fn chars(text: &str) -> usize {
    text.chars().count()
}

// Like Symfony's `Terminal`, which Laravel uses: `COLUMNS`, then `stty size`, then 80.
fn terminal_width() -> usize {
    let columns = env::var("COLUMNS")
        .ok()
        .and_then(|columns| columns.parse().ok());
    columns.or_else(stty_columns).unwrap_or(80)
}

fn stty_columns() -> Option<usize> {
    let tty = File::open("/dev/tty").ok()?;
    let output = Process::new("stty")
        .arg("size")
        .stdin(tty)
        .stderr(Stdio::null())
        .output()
        .ok()?;
    let size = String::from_utf8(output.stdout).ok()?;
    size.split_whitespace().nth(1)?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(method: Method, path: &str, name: Option<&str>, action: &'static str) -> RouteRow {
        RouteRow {
            method,
            path: path.to_owned(),
            name: name.map(str::to_owned),
            action,
        }
    }

    fn blog_rows() -> Vec<RouteRow> {
        let show = "bootstrap::app::http::controllers::post_controller::PostController::show";
        vec![
            row(
                Method::POST,
                "/api/posts/{id}/like",
                None,
                "bootstrap::routes::{{closure}}",
            ),
            row(Method::GET, "/api/posts/{id}", Some("posts.show"), show),
            row(
                Method::GET,
                "/",
                Some("home"),
                "bootstrap::routes::web::home",
            ),
        ]
    }

    #[test]
    fn routes_are_listed_by_path_with_dots_to_the_width() {
        let lines = table(&blog_rows(), 80, Paint(false));

        assert_eq!(
            lines,
            [
                "",
                "  GET|HEAD       / .................................. home › routes::web::home",
                "  GET|HEAD       api/posts/{id} ............. posts.show › PostController@show",
                "  POST           api/posts/{id}/like .........................................",
                "",
                "                                                            Showing [3] routes",
                "",
            ]
        );
    }

    #[test]
    fn a_line_too_long_for_the_terminal_ends_its_action_with_an_ellipsis() {
        let lines = table(&blog_rows(), 50, Paint(false));

        assert_eq!(lines[2], "  GET|HEAD       api/posts/{id} posts.show › Po…");
    }

    #[test]
    fn colors_mark_get_blue_post_yellow_and_parameters_yellow() {
        let lines = table(&blog_rows(), 80, Paint(true));

        assert!(
            lines[1].contains("\x1b[34mGET\x1b[0m\x1b[90m|\x1b[0m\x1b[90mHEAD"),
            "{:?}",
            lines[1]
        );
        assert!(lines[3].contains("\x1b[33mPOST\x1b[0m"), "{:?}", lines[3]);
        assert!(lines[2].contains("\x1b[33m{id}\x1b[0m"), "{:?}", lines[2]);
    }

    #[test]
    fn handlers_are_named_by_controller_and_method() {
        let method = "bootstrap::app::http::controllers::post_controller::PostController::index";

        assert_eq!(
            short_action(method).as_deref(),
            Some("PostController@index")
        );
        assert_eq!(
            short_action("bootstrap::routes::api::health").as_deref(),
            Some("routes::api::health")
        );
        assert_eq!(
            short_action("bootstrap::routes::web::routes::{{closure}}"),
            None
        );
    }
}
