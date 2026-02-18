use crate::validation_rules::types::ActionRule;

pub fn get_action_rule(model: &str, action: &str) -> Option<ActionRule<'static>> {
    match model {
        "customer" => crate::models::customer::get_action_rule(action),
        "company" => crate::models::company::get_action_rule(action),
        "quote" => crate::models::quote::get_action_rule(action),
        "invoice" => crate::models::invoice::get_action_rule(action),
        "sale" => crate::models::sale::get_action_rule(action),
        "blog" => crate::models::blog::get_action_rule(action),
        "interaction" => crate::models::interaction::get_action_rule(action),
        "certificate" => crate::models::certificate::get_action_rule(action),
        "lead" => crate::models::lead::get_action_rule(action),
        "book" => crate::models::book::get_action_rule(action),
        "fahrrad" => crate::models::fahrrad::get_action_rule(action),
        "appointment" => crate::models::appointment::get_action_rule(action),
        "category" => crate::models::category::get_action_rule(action),
        "product" => crate::models::product::get_action_rule(action),
        "tag" => crate::models::tag::get_action_rule(action),
        "review" => crate::models::review::get_action_rule(action),
        "comment" => crate::models::comment::get_action_rule(action),
        "contact" => crate::models::contact::get_action_rule(action),
        "company_invite" => crate::models::company_invite::get_action_rule(action),
        "profile" => crate::models::profile::get_action_rule(action),
        "user" => crate::models::user::get_action_rule(action),
        "template" => crate::models::template::get_action_rule(action),
        "plan" => crate::models::plan::get_action_rule(action),
        "timeline" => crate::models::timeline::get_action_rule(action),
        "curriculum" => crate::models::curriculum::get_action_rule(action),
        "cv" => crate::models::cv::get_action_rule(action),
        "setting" => crate::models::setting::get_action_rule(action),
        _ => None,
    }
}
