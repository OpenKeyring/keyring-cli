fn main() {
    let password = "MyStr0ng!P@ssw0rd#2024";
    let chars: Vec<char> = password.chars().collect();

    println!("Checking: {}", password);
    println!("Length: {}", password.len());

    // Check for sequential characters (4+ window)
    for (i, window) in chars.windows(4).enumerate() {
        let sequential = window.iter().enumerate().all(|(j, &c)| {
            if j == 0 { return true; }
            let prev = window[j - 1] as i32;
            let curr = c as i32;
            let diff = (curr - prev).abs();
            diff == 1 || diff == 2
        });
        if sequential {
            println!("Sequential found at {}: {:?}", i, window);
        }
    }

    // Check unique char ratio
    let unique_chars: std::collections::HashSet<char> = password.chars().collect();
    let ratio = unique_chars.len() as f64 / password.len() as f64;
    println!("Unique chars: {}/{} = {:.2}", unique_chars.len(), password.len(), ratio);
}
