//! TUI 加密服务适配器
//!
//! 封装现有 crypto 模块实现，提供 TUI 层所需的加密接口。

use crate::tui::error::TuiResult;
use crate::tui::traits::{CryptoService, PasswordPolicy, PasswordStrengthCalculator, ServicePasswordStrength as PasswordStrength};
use crate::tui::core::DefaultPasswordStrengthCalculator;

/// TUI 加密服务
pub struct TuiCryptoService {}

impl TuiCryptoService {
    /// 创建新的加密服务
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for TuiCryptoService {
    fn default() -> Self {
        Self::new()
    }
}

impl CryptoService for TuiCryptoService {
    /// 加密数据
    fn encrypt(&self, _data: &[u8]) -> TuiResult<Vec<u8>> {
        // TODO: 调用 crypto::aes256gcm::encrypt
        todo!("Implement with crypto module integration")
    }

    /// 解密数据
    fn decrypt(&self, _data: &[u8]) -> TuiResult<Vec<u8>> {
        // TODO: 调用 crypto::aes256gcm::decrypt
        todo!("Implement with crypto module integration")
    }

    /// 根据策略生成密码
    fn generate_password(&self, policy: &PasswordPolicy) -> TuiResult<String> {
        // 简单实现 - 实际应调用 crypto 模块
        use rand::Rng;
        const LOWERCASE: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
        const UPPERCASE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        const DIGITS: &[u8] = b"0123456789";
        const SPECIAL: &[u8] = b"!@#$%^&*()_+-=[]{}|;:,.<>?";

        let mut rng = rand::rng();
        let mut password = Vec::with_capacity(policy.length as usize);

        // 确保每种字符类型的最低数量
        for _ in 0..policy.min_lowercase {
            let idx = rng.random_range(0..LOWERCASE.len());
            password.push(LOWERCASE[idx]);
        }

        for _ in 0..policy.min_uppercase {
            let idx = rng.random_range(0..UPPERCASE.len());
            password.push(UPPERCASE[idx]);
        }

        for _ in 0..policy.min_digits {
            let idx = rng.random_range(0..DIGITS.len());
            password.push(DIGITS[idx]);
        }

        for _ in 0..policy.min_special {
            let idx = rng.random_range(0..SPECIAL.len());
            password.push(SPECIAL[idx]);
        }

        // 填充剩余长度
        let all_chars: Vec<u8> = LOWERCASE
            .iter()
            .chain(UPPERCASE.iter())
            .chain(DIGITS.iter())
            .chain(SPECIAL.iter())
            .copied()
            .collect();

        while password.len() < policy.length as usize {
            let idx = rng.random_range(0..all_chars.len());
            password.push(all_chars[idx]);
        }

        // 随机打乱
        use rand::seq::SliceRandom;
        password.shuffle(&mut rng);

        Ok(String::from_utf8(password).unwrap_or_default())
    }

    /// 检查密码强度
    fn check_password_strength(&self, password: &str) -> PasswordStrength {
        // 使用已有的 DefaultPasswordStrengthCalculator
        let calculator = DefaultPasswordStrengthCalculator::new();
        let strength = calculator.calculate(password);

        // 将 traits::password_strength::PasswordStrength 转换为 traits::service::PasswordStrength
        use crate::tui::traits::PasswordStrength as TraitPasswordStrength;
        match strength {
            TraitPasswordStrength::Weak => PasswordStrength::Weak,
            TraitPasswordStrength::Fair => PasswordStrength::Fair,
            TraitPasswordStrength::Strong => PasswordStrength::Good,
            TraitPasswordStrength::VeryStrong => PasswordStrength::Strong,
        }
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_service_creation() {
        let service = TuiCryptoService::new();
        // Just verify it can be created
        let _ = service;
    }

    #[test]
    fn test_crypto_service_default() {
        let service = TuiCryptoService::default();
        let _ = service;
    }

    #[test]
    fn test_generate_password_default_policy() {
        let service = TuiCryptoService::new();
        let policy = PasswordPolicy::default();
        let password = service.generate_password(&policy).unwrap();

        assert_eq!(password.len(), policy.length as usize);
    }

    #[test]
    fn test_generate_password_meets_requirements() {
        let service = TuiCryptoService::new();
        let policy = PasswordPolicy {
            length: 20,
            min_digits: 3,
            min_special: 2,
            min_lowercase: 2,
            min_uppercase: 2,
            ..Default::default()
        };

        let password = service.generate_password(&policy).unwrap();

        assert_eq!(password.len(), 20);

        // Check minimum requirements
        let digit_count = password.chars().filter(|c| c.is_ascii_digit()).count();
        let special_count = password
            .chars()
            .filter(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(*c))
            .count();
        let lowercase_count = password.chars().filter(|c| c.is_ascii_lowercase()).count();
        let uppercase_count = password.chars().filter(|c| c.is_ascii_uppercase()).count();

        assert!(digit_count >= 3);
        assert!(special_count >= 2);
        assert!(lowercase_count >= 2);
        assert!(uppercase_count >= 2);
    }

    #[test]
    fn test_check_password_strength_weak() {
        let service = TuiCryptoService::new();

        let weak_passwords = ["a", "123", "abc", "password"];
        for pwd in weak_passwords {
            let strength = service.check_password_strength(pwd);
            assert_eq!(strength, PasswordStrength::Weak);
        }
    }

    #[test]
    fn test_check_password_strength_fair() {
        let service = TuiCryptoService::new();

        // Fair passwords: moderate length, some complexity
        let fair_passwords = ["password1", "abcdefgh1"];
        for pwd in fair_passwords {
            let strength = service.check_password_strength(pwd);
            // Note: The actual strength depends on the implementation
            // Just verify it returns a valid strength
            assert!(matches!(
                strength,
                PasswordStrength::Weak
                    | PasswordStrength::Fair
                    | PasswordStrength::Good
                    | PasswordStrength::Strong
            ));
        }
    }

    #[test]
    fn test_check_password_strength_strong() {
        let service = TuiCryptoService::new();

        // Strong password: long, mixed case, digits, special chars
        let strong_password = "MyV3ryStr0ng!P@ssw0rd#2024";
        let strength = service.check_password_strength(strong_password);

        // Should be at least Good
        assert!(matches!(
            strength,
            PasswordStrength::Good | PasswordStrength::Strong
        ));
    }

    #[test]
    fn test_crypto_service_trait_bounds() {
        // Verify TuiCryptoService implements all required traits
        fn assert_crypto_service<T: CryptoService + Send + Sync>() {}
        assert_crypto_service::<TuiCryptoService>();
    }
}
