//! Password generation functions
//!
//! Provides random, memorable, and PIN generation capabilities.

use crate::error::{KeyringError, Result};
use rand::prelude::IndexedRandom;
use rand::Rng;

/// Generate a random password with specified length and character set
///
/// # Arguments
/// * `length` - The desired password length
/// * `numbers` - Whether to include numeric characters
/// * `symbols` - Whether to include special symbols
///
/// # Returns
/// A randomly generated password string
///
/// # Character Sets
/// - Lowercase: "abcdefghijkmnpqrstuvwxyz" (excludes: o, l)
/// - Uppercase: "ABCDEFGHJKLMNPQRSTUVWXYZ" (excludes: O, I)
/// - Numbers: "23456789" (excludes: 0, 1)
/// - Symbols: "!@#$%^&*()_+-=[]{}|;:,.<>?"
pub fn generate_random(length: usize, numbers: bool, symbols: bool) -> Result<String> {
    if length < 4 {
        return Err(KeyringError::InvalidInput {
            context: "Password length must be at least 4 characters".to_string(),
        });
    }
    if length > 128 {
        return Err(KeyringError::InvalidInput {
            context: "Password length cannot exceed 128 characters".to_string(),
        });
    }

    // Character sets excluding ambiguous characters
    let lowercase = "abcdefghijkmnpqrstuvwxyz"; // no o, l
    let uppercase = "ABCDEFGHJKLMNPQRSTUVWXYZ"; // no O, I
    let nums = "23456789"; // no 0, 1
    let syms = "!@#$%^&*"; // safe symbols only (no : ; , .)

    let mut charset = String::from(lowercase);
    charset.push_str(uppercase);

    if numbers {
        charset.push_str(nums);
    }
    if symbols {
        charset.push_str(syms);
    }

    let chars: Vec<char> = charset.chars().collect();
    let nums_chars: Vec<char> = nums.chars().collect();
    let syms_chars: Vec<char> = syms.chars().collect();
    let mut rng = rand::rng();

    // Build password ensuring required character types are included
    let mut password_chars: Vec<char> = Vec::with_capacity(length);

    // First, ensure at least one of each required type
    if numbers {
        let idx = rng.random_range(0..nums_chars.len());
        password_chars.push(nums_chars[idx]);
    }
    if symbols {
        let idx = rng.random_range(0..syms_chars.len());
        password_chars.push(syms_chars[idx]);
    }

    // Fill remaining length with random characters from the full charset
    while password_chars.len() < length {
        let idx = rng.random_range(0..chars.len());
        password_chars.push(chars[idx]);
    }

    // Shuffle to avoid predictable patterns (required chars at the start)
    use rand::seq::SliceRandom;
    password_chars.shuffle(&mut rng);

    Ok(password_chars.into_iter().collect())
}

/// Generate a memorable password using word-based approach
///
/// # Arguments
/// * `word_count` - The number of words to include (3-12)
///
/// # Returns
/// A memorable passphrase with capitalized words (e.g., "Correct-Horse-Battery-Staple")
pub fn generate_memorable(word_count: usize) -> Result<String> {
    const WORDS: &[&str] = &[
        "correct",
        "horse",
        "battery",
        "staple",
        "apple",
        "banana",
        "cherry",
        "dragon",
        "elephant",
        "flower",
        "garden",
        "house",
        "island",
        "jungle",
        "kangaroo",
        "lemon",
        "mountain",
        "nectar",
        "orange",
        "piano",
        "queen",
        "river",
        "sunshine",
        "tiger",
        "umbrella",
        "violet",
        "whale",
        "xylophone",
        "yellow",
        "zebra",
        "castle",
        "desert",
        "eagle",
        "forest",
        "giraffe",
        "harbor",
        "igloo",
        "journey",
        "kingdom",
        "lantern",
        "meadow",
        "night",
        "ocean",
        "planet",
        "quartz",
        "rainbow",
        "star",
        "tower",
        "universe",
        "valley",
        "wave",
        "crystal",
        "year",
        "zen",
        "bridge",
        "cloud",
        "diamond",
        "emerald",
        "fountain",
        "galaxy",
        "horizon",
        "infinity",
        "jewel",
    ];

    if word_count < 3 {
        return Err(KeyringError::InvalidInput {
            context: "Word count must be at least 3".to_string(),
        });
    }
    if word_count > 12 {
        return Err(KeyringError::InvalidInput {
            context: "Word count cannot exceed 12".to_string(),
        });
    }

    let mut rng = rand::rng();
    let selected: Vec<&str> = WORDS
        .choose_multiple(&mut rng, word_count)
        .copied()
        .collect();

    // Capitalize first letter of each word and join with hyphens
    let password = selected
        .iter()
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join("-");

    Ok(password)
}

/// Generate a numeric PIN
///
/// # Arguments
/// * `length` - The desired PIN length (4-16)
///
/// # Returns
/// A numeric PIN using digits 2-9 only (excludes 0 and 1 for clarity)
pub fn generate_pin(length: usize) -> Result<String> {
    if length < 4 {
        return Err(KeyringError::InvalidInput {
            context: "PIN length must be at least 4 digits".to_string(),
        });
    }
    if length > 16 {
        return Err(KeyringError::InvalidInput {
            context: "PIN length cannot exceed 16 digits".to_string(),
        });
    }

    // Use only 2-9 to avoid ambiguous 0 and 1
    let digits = [b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9'];
    let mut rng = rand::rng();
    let pin: String = (0..length)
        .map(|_| {
            let idx = rng.random_range(0..digits.len());
            digits[idx] as char
        })
        .collect();

    Ok(pin)
}
