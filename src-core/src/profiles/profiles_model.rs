use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProfileSharingRule {
    #[default]
    Private,
    FamilyReadOnly,
    FamilyReadWrite,
}

impl ProfileSharingRule {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProfileSharingRule::Private => "private",
            ProfileSharingRule::FamilyReadOnly => "family_read_only",
            ProfileSharingRule::FamilyReadWrite => "family_read_write",
        }
    }

    pub fn from_str(value: &str) -> Self {
        match value {
            "family_read_only" => Self::FamilyReadOnly,
            "family_read_write" => Self::FamilyReadWrite,
            _ => Self::Private,
        }
    }
}

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::profiles)]
pub struct Profile {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing)]
    pub password_hash: Option<String>,
    #[serde(skip_serializing)]
    pub encryption_salt: Option<String>,
    #[serde(skip_serializing)]
    pub encryption_key: Option<String>,
    pub sharing_rule: String,
    pub family_mode_enabled: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::profiles)]
pub struct NewProfile<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub password_hash: Option<&'a str>,
    pub encryption_salt: Option<&'a str>,
    pub encryption_key: Option<&'a str>,
    pub sharing_rule: &'a str,
    pub family_mode_enabled: bool,
    pub created_at: NaiveDateTime,
}

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = crate::schema::profile_shares)]
pub struct ProfileShare {
    pub id: String,
    pub owner_profile_id: String,
    pub shared_profile_id: String,
    pub resource_type: String,
    pub resource_id: String,
    pub permissions: String,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::profile_shares)]
pub struct NewProfileShare<'a> {
    pub id: &'a str,
    pub owner_profile_id: &'a str,
    pub shared_profile_id: &'a str,
    pub resource_type: &'a str,
    pub resource_id: &'a str,
    pub permissions: &'a str,
    pub created_at: NaiveDateTime,
}
