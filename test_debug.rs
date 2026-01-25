fn calculate_strength(password: &str) -> u8 {
    let mut score = 0u8;

    // 1. Length scoring
    let length_score = match password.len() {
        0..=7 => (password.len() * 3) as u8,
        8..=11 => 25,
        12..=15 => 32,
        16..=19 => 38,
        _ => 40,
    };
    score += length_score;
    eprintln!("After length: {}", score);

    // 2. Character variety
    let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
    let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_symbol = password.chars().any(|c| !c.is_alphanumeric());

    let variety_count = [has_lower, has_upper, has_digit, has_symbol]
        .iter()
        .filter(|&&x| x)
        .count();

    let variety_score = match variety_count {
        1 => 5,
        2 => 12,
        3 => 20,
        4 => 30,
        _ => 0,
    };
    score += variety_score;
    eprintln!("After variety: {}", score);

    // 4. Common pattern penalties
    let password_lower = password.to_lowercase();

    let common_patterns = [
        "password", "qwerty", "asdfgh", "zxcvbn",
        "letmein", "welcome", "login", "admin",
        "123456", "111111", "123123",
    ];

    for pattern in &common_patterns {
        if password_lower.contains(pattern) {
            eprintln!("Found common pattern: {}", pattern);
            score = score.saturating_sub(25);
            break;
        }
    }

    // 5. Bonus for length > 16
    if password.len() > 16 {
        score += 5;
    }

    // 6. Bonus for unique characters
    let unique_chars: std::collections::HashSet<char> = password.chars().collect();
    if unique_chars.len() as f64 / password.len() as f64 > 0.7 {
        score += 5;
    }

    eprintln!("Final score: {}", score);
    score.max(0).min(100)
}

fn main() {
    let result = calculate_strength("MyPass123!");
    eprintln!("Result: {}", result);
}
