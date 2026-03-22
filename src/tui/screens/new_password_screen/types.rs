//! Type definitions for NewPasswordScreen
//!
//! Contains FormField enum and NewPasswordRecord struct.

/// Form field identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormField {
    Name,
    Username,
    PasswordType,
    PasswordLength,
    Password,
    Url,
    Notes,
    Tags,
    Group,
}

impl FormField {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Name => "Name",
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

    pub fn is_required(&self) -> bool {
        matches!(self, Self::Name)
    }

    pub fn index(&self) -> usize {
        match self {
            Self::Name => 0,
            Self::Username => 1,
            Self::PasswordType => 2,
            Self::PasswordLength => 3,
            Self::Password => 4,
            Self::Url => 5,
            Self::Notes => 6,
            Self::Tags => 7,
            Self::Group => 8,
        }
    }

    pub fn from_index(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Self::Name),
            1 => Some(Self::Username),
            2 => Some(Self::PasswordType),
            3 => Some(Self::PasswordLength),
            4 => Some(Self::Password),
            5 => Some(Self::Url),
            6 => Some(Self::Notes),
            7 => Some(Self::Tags),
            8 => Some(Self::Group),
            _ => None,
        }
    }
}

/// Password record created from the form
#[derive(Debug, Clone)]
pub struct NewPasswordRecord {
    pub id: uuid::Uuid,
    pub name: String,
    pub username: Option<String>,
    pub password: String,
    pub url: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub group: String,
}
