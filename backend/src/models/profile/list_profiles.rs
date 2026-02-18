use crate::validation_rules::types::{common_rules, empty_coercions, ActionRule};

pub fn rule() -> ActionRule<'static> {
    ActionRule {
        model: "Profile",
        action: "list_profiles",
        version: "1",
        fields: Vec::new(),
        coercions: empty_coercions(),
        rules: common_rules(),
    }
}
