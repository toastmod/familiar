use std::{error::Error, io::Write};

/// Reads the current contents of the system clipboard.
///
/// Returns an error if the clipboard cannot be accessed.
fn get_clipboard_contents() -> Result<String, Box<dyn Error>> {
    Ok(clipboard_win::get_clipboard_string()?)
}

/// Writes the given text to the system clipboard.
///
/// Returns an error if the clipboard cannot be written to.
fn set_clipboard_contents(text: &str) -> Result<(), Box<dyn Error>> {
    clipboard_win::set_clipboard_string(text)?;
    Ok(())
}

/// Reads a single line of input from the console (the user's instructions).
///
/// Returns an error if no input was provided (e.g. stdin was closed) or reading
/// failed.
fn read_instructions() -> Result<String, Box<dyn Error>> {
    print!("> ");
    std::io::stdout().flush()?;
    let mut input = String::new();
    match std::io::stdin().read_line(&mut input) {
        Ok(0) => {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "stdin closed without input",
            )))
        }
        Ok(_) => Ok(input.trim().to_string()),
        Err(e) => Err(Box::new(e)),
    }
}

/// Sends a Chat Completions request to the given endpoint using the OpenAI JSON
/// schema and returns the raw assistant text.
///
/// The endpoint is expected to accept a `POST` with a JSON body of the OpenAI
/// Chat Completions shape. Authentication is left to the caller (e.g. a local
/// public endpoint), so no API key is attached.
///
/// Returns an error if the request fails or the response cannot be parsed.
fn chat_completion(url: &str, model: &str, prompt: &str) -> Result<String, Box<dyn Error>> {
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
        .send()?
        .error_for_status()?
        .json::<serde_json::Value>()?;

    let choices = response.get("choices").ok_or("missing 'choices'")?;
    let first = choices.get(0).ok_or("empty 'choices'")?;
    let message = first.get("message").ok_or("missing 'message'")?;
    let content = message.get("content").ok_or("missing 'content'")?;
    content
        .as_str()
        .ok_or("content is not a string".to_string().into())
        .map(|s| s.to_string())
}

fn main() -> Result<(), Box<dyn Error>> {
    loop {
        // Ask the user how they want the clipboard contents edited.
        let instructions = read_instructions()?;

        let text = get_clipboard_contents()?;

        // Example: send a Chat Completions request to a local endpoint.
        let prompt = format!("<instruction>{instructions}</instruction> \"{text}\"");
        let result = chat_completion(
            "http://localhost:10000/v1/chat/completions",
            "gpt-4o-mini",
            &prompt,
        );
        let text = result?;

        set_clipboard_contents(&text)?;
        println!("Done!");
    }

    Ok(())
}
