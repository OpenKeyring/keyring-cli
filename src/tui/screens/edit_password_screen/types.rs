//! Type definitions for EditPasswordScreen
//!
//! Contains EditFormField enum and EditedPasswordFields struct.

/// Form field identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EditFormField {
    Username,
    PasswordType,
    PasswordLength,
    Password,
    Url,
    Notes,
    Tags,
    Group,
}

impl EditFormField {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Username => "Username",
            Self::PasswordType => "Password Type",
            Self::PasswordLength => "Length",
            Self::Password => "Password",
            Self::Url => "URL",
            Self::Notes => "Notes",
            Self::Tags => "Tags",
            Self::Group => "Group",
        }
    }

    pub fn index(&self) -> usize {
        match self {
            Self::Username => 0,
            Self::PasswordType => 1,
            Self::PasswordLength => 2,
            Self::Password => 3,
            Self::Url => 4,
            Self::Notes => 5,
            Self::Tags => 6,
            Self::Group => 7,
        }
    }

    pub fn from_index(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Self::Username),
            1 => Some(Self::PasswordType),
            2 => Some(Self::PasswordLength),
            3 => Some(Self::Password),
            4 => Some(Self::Url),
            5 => Some(Self::Notes),
            6 => Some(Self::Tags),
            7 => Some(Self::Group),
            _ => None,
        }
    }
}

/// Edited password fields result
#[derive(Debug, Clone)]
pub struct EditedPasswordFields {
    pub id: uuid::Uuid,
    pub name: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub group_id: Option<String>,
}
