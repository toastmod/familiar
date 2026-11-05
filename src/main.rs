/// Reads the current contents of the system clipboard.
///
/// Returns `None` if the clipboard is empty or cannot be accessed.
fn get_clipboard_contents() -> Option<String> {
    clipboard_win::get_clipboard_string().ok()
}

fn main() {
    match get_clipboard_contents() {
        Some(text) => println!("Clipboard contents:\n{text}"),
        None => println!("Clipboard is empty or unavailable."),
    }
}
