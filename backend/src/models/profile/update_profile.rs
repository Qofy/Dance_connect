use crate::validation_rules::types::{common_rules, ActionRule};

pub fn rule() -> ActionRule<'static> {
    ActionRule {
        model: "Profile",
        action: "update_profile",
        version: "1",
        fields: super::fields_for_action("update_profile"),
        coercions: super::coercions(),
        rules: common_rules(),
    }
}
