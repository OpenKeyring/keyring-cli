fn calculate_strength(password: &str) -> u8 {
    let mut score = 0u8;

    // 1. Length scoring (up to 40 points)
    let length_score = match password.len() {
        0..=7 => (password.len() * 3) as u8,
        8..=11 => 25,
        12..=15 => 32,
        16..=19 => 38,
        _ => 40,
    };
    score += length_score;
    println!("{}: len={}, length_score={}", password, password.len(), length_score);

    // 2. Character variety (up to 30 points)
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
    println!("{}: variety_count={}, variety_score={}", password, variety_count, variety_score);

    // 5. Bonus for length > 16
    if password.len() > 16 {
        score += 5;
        println!("{}: added >16 bonus +5", password);
    }

    // 6. Bonus for unique characters
    let unique_chars: std::collections::HashSet<char> = password.chars().collect();
    if unique_chars.len() as f64 / password.len() as f64 > 0.7 {
        score += 5;
        println!("{}: added unique bonus +5", password);
    }

    println!("{}: final_score={}", password, score);
    score.max(0).min(100)
}

fn main() {
    println!("MyPass123! = {}", calculate_strength("MyPass123!"));
    println!("MyStr0ng!P@ssw0rd#2024 = {}", calculate_strength("MyStr0ng!P@ssw0rd#2024"));
}
