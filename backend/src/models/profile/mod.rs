use crate::validation_rules::types::{ActionRule, Coercions, FieldRule};

pub mod delete_profile;
pub mod get_profile;
pub mod list_profiles;
pub mod new_profile;
pub mod update_profile;

fn profile_coercions() -> Coercions<'static> {
    Coercions {
        empty_string_is_null: true,
        numeric_string_to_number: &[],
    }
}

fn profile_fields() -> Vec<FieldRule<'static>> {
    vec![
        FieldRule {
            name: "full_name",
            r#type: "string",
            required: false,
            nullable: true,
            pii: true,
            min_len: None,
            max_len: Some(120),
            pattern: Some(r"^[A-Za-z .,'\-]{0,120}$"),
            r#enum: None,
            format: None,
            example: Some("Jordan Lee"),
            ui: None,
        },
        FieldRule {
            name: "photo_url",
            r#type: "string",
            required: false,
            nullable: true,
            pii: false,
            min_len: None,
            max_len: Some(2048),
            pattern: None,
            r#enum: None,
            format: Some("url"),
            example: None,
            ui: None,
        },
        FieldRule {
            name: "phone",
            r#type: "string",
            required: false,
            nullable: true,
            pii: true,
            min_len: Some(6),
            max_len: Some(32),
            pattern: Some(r"^[0-9+()\-\.\s]{6,32}$"),
            r#enum: None,
            format: Some("phone"),
            example: None,
            ui: None,
        },
        FieldRule {
            name: "secondary_email",
            r#type: "string",
            required: false,
            nullable: true,
            pii: true,
            min_len: None,
            max_len: Some(254),
            pattern: None,
            r#enum: None,
            format: Some("email"),
            example: None,
            ui: None,
        },
        FieldRule {
            name: "preferences",
            r#type: "object",
            required: false,
            nullable: true,
            pii: false,
            min_len: None,
            max_len: None,
            pattern: None,
            r#enum: None,
            format: None,
            example: None,
            ui: None,
        },
        FieldRule {
            name: "preferences.language",
            r#type: "string",
            required: false,
            nullable: true,
            pii: false,
            min_len: None,
            max_len: None,
            pattern: Some(r"^[a-z]{2}(-[A-Z]{2})?$"),
            r#enum: None,
            format: Some("language-code"),
            example: Some("en"),
            ui: None,
        },
    ]
}

pub fn get_action_rule(action: &str) -> Option<ActionRule<'static>> {
    match action {
        "new_profile" => Some(new_profile::rule()),
        "update_profile" => Some(update_profile::rule()),
        "delete_profile" => Some(delete_profile::rule()),
        "list_profiles" => Some(list_profiles::rule()),
        "get_profile" => Some(get_profile::rule()),
        _ => None,
    }
}

pub(crate) fn fields_for_action(_action: &str) -> Vec<FieldRule<'static>> {
    profile_fields()
}

pub(crate) fn coercions() -> Coercions<'static> {
    profile_coercions()
}
