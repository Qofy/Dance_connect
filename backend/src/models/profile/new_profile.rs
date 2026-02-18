use crate::validation_rules::types::{common_rules, ActionRule};

pub fn rule() -> ActionRule<'static> {
    ActionRule {
        model: "Profile",
        action: "new_profile",
        version: "1",
        fields: super::fields_for_action("new_profile"),
        coercions: super::coercions(),
        rules: common_rules(),
    }
}
