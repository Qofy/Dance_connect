use crate::validation_rules::types::{common_rules, empty_coercions, ActionRule};

pub fn rule() -> ActionRule<'static> {
    ActionRule {
        model: "Profile",
        action: "get_profile",
        version: "1",
        fields: Vec::new(),
        coercions: empty_coercions(),
        rules: common_rules(),
    }
}
