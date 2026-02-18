use crate::validation_rules::types::ActionRule;

pub mod delete_user;
pub mod get_user;
pub mod list_users;
pub mod new_user;
pub mod update_user;

pub fn get_action_rule(action: &str) -> Option<ActionRule<'static>> {
    match action {
        "new_user" => Some(new_user::rule()),
        "update_user" => Some(update_user::rule()),
        "delete_user" => Some(delete_user::rule()),
        "list_users" => Some(list_users::rule()),
        "get_user" => Some(get_user::rule()),
        _ => None,
    }
}
