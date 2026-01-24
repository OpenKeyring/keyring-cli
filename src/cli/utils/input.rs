use std::io::{self, Write};
use rpassword::read_password;

pub fn prompt_for_password(prompt: &str) -> io::Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;

    read_password().map(|mut s| {
        s.push('\n'); // Add newline for consistency
        s
    })
}

pub fn prompt_for_password_confirm(prompt: &str, confirm_prompt: &str) -> io::Result<String> {
    let password1 = prompt_for_password(prompt)?;
    let password2 = prompt_for_password(confirm_prompt)?;

    if password1 != password2 {
        return Err(io::Error::new(io::ErrorKind::Other, "Passwords do not match"));
    }

    Ok(password1.trim().to_string())
}

pub fn prompt_for_confirmation(prompt: &str) -> io::Result<bool> {
    print!("{} (y/N): ", prompt);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let input = input.trim().to_lowercase();
    Ok(input == "y" || input == "yes")
}

pub fn prompt_for_input(prompt: &str, required: bool) -> io::Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let input = input.trim().to_string();

    if required && input.is_empty() {
        return Err(io::Error::new(io::ErrorKind::Other, "Input is required"));
    }

    Ok(input)
}