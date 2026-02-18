use crate::validation_rules::types::{common_rules, empty_coercions, ActionRule};

pub fn rule() -> ActionRule<'static> {
    ActionRule {
        model: "User",
        action: "list_users",
        version: "1",
        fields: Vec::new(),
        coercions: empty_coercions(),
        rules: common_rules(),
    }
}
