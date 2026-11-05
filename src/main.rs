use std::{error::Error, io::Write};

/// Reads the current contents of the system clipboard.
///
/// Returns `None` if the clipboard is empty or cannot be accessed.
fn get_clipboard_contents() -> Option<String> {
    clipboard_win::get_clipboard_string().ok()
}

/// Writes the given text to the system clipboard.
///
/// Returns `false` if the clipboard cannot be written to.
fn set_clipboard_contents(text: &str) -> bool {
    clipboard_win::set_clipboard_string(text).is_ok()
}

/// Reads a single line of input from the console (the user's instructions).
///
/// Returns `None` if no input was provided (e.g. stdin was closed) or reading
/// failed.
fn read_instructions() -> Option<String> {
    print!("> ");
    std::io::stdout().flush().ok()?;
    let mut input = String::new();
    match std::io::stdin().read_line(&mut input) {
        Ok(0) => None, // stdin closed without input
        Ok(_) => Some(input.trim().to_string()),
        Err(_) => None,
    }
}

/// Sends a Chat Completions request to the given endpoint using the OpenAI JSON
/// schema and returns the raw assistant text.
///
/// The endpoint is expected to accept a `POST` with a JSON body of the OpenAI
/// Chat Completions shape. Authentication is left to the caller (e.g. a local
/// public endpoint), so no API key is attached.
///
/// Returns `None` if the request fails or the response cannot be parsed.
fn chat_completion(url: &str, model: &str, prompt: &str) -> Option<String> {
    let body = serde_json::json!({
        "model": model,
        "messages": [
            { "role": "system", "content": "You are a helpful assistant that edits text." },
            { "role": "user", "content": prompt }
        ],
        "temperature": 0.7
    });

    let response = reqwest::blocking::Client::new()
        .post(url)
        .json(&body)
        .send()
        .ok()?
        .error_for_status()
        .ok()?
        .json::<serde_json::Value>()
        .ok()?;

    response.get("choices").and_then(|choices| choices.get(0))?.get("message")?.get("content").and_then(|c| c.as_str()).map(|s| s.to_string())
}

fn main() {
    // Ask the user how they want the clipboard contents edited.
    let instructions = read_instructions().unwrap();

    match get_clipboard_contents() {
        Some(text) => {
            // Example: send a Chat Completions request to a local endpoint.
            let prompt = format!("<instruction>{instructions}</instruction> \"{text}\"");
            match chat_completion("http://localhost:10000/v1/chat/completions", "gpt-4o-mini", &prompt) {
                Some(text) => {
                    set_clipboard_contents(&text);
                    println!("Done!");
                },
                None => println!("\nChat completion failed."),
            }
        }
        None => println!("Clipboard is empty or unavailable."),
    }

}
