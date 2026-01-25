fn main() {
    let pwd = "CorrectHorseBattery!Staple#2024";
    println!("Length: {}", pwd.len());

    let password_lower = pwd.to_lowercase();
    let common_patterns = [
        "password", "qwerty", "asdfgh", "zxcvbn",
        "letmein", "welcome", "login", "admin",
        "123456", "111111", "123123",
    ];

    for pattern in &common_patterns {
        if password_lower.contains(pattern) {
            println!("Contains pattern: {}", pattern);
        }
    }

    // Check for repeated chars
    let chars: Vec<char> = pwd.chars().collect();
    for window in chars.windows(3) {
        if window.iter().all(|&c| c == window[0]) {
            println!("Repeated: {:?}", window);
        }
    }

    // Check for sequential (4+)
    for window in chars.windows(4) {
        let sequential = window.iter().enumerate().all(|(i, &c)| {
            if i == 0 { return true; }
            let prev = window[i - 1] as i32;
            let curr = c as i32;
            curr - prev == 1
        });
        let reverse = window.iter().enumerate().all(|(i, &c)| {
            if i == 0 { return true; }
            let prev = window[i - 1] as i32;
            let curr = c as i32;
            prev - curr == 1
        });
        if sequential || reverse {
            println!("Sequential: {:?}", window);
        }
    }
}
