pub fn format_chat(sender: &str, msg: &str) -> String {
    format!("\x1b[94m[CHAT:\x1b[3m{}\x1b[23m]\x1b[0m {}", sender, msg)
}

pub fn format_log(style: &str, name: &str, msg: &str) -> String {
    format!("\x1b[{}m[{}]\x1b[0m {}", style, name, msg)
}
