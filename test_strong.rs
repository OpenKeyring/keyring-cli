fn check_substitutions(password: &str) -> bool {
    let password_lower = password.to_lowercase();
    let common_patterns = [
        "password", "qwerty", "asdfgh", "zxcvbn",
        "letmein", "welcome", "login", "admin",
        "123456", "111111", "123123",
    ];

    let substitutions = [
        ("@", "a"), ("0", "o"), ("3", "e"), ("1", "i"),
        ("$", "s"), ("7", "t"), ("9", "g"),
    ];

    for (sub, orig) in &substitutions {
        if password_lower.contains(sub) {
            let subbed_with = password_lower.replace(sub, orig);
            if common_patterns.iter().any(|p| subbed_with.contains(p)) {
                return true;
            }
        }
    }
    false
}

fn main() {
    let pwd = "MyStr0ng!P@ssw0rd#2024";
    println!("Checking: {}", pwd);
    println!("Has substitution pattern: {}", check_substitutions(pwd));
}
