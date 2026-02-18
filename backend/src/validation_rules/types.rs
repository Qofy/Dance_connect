use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct EnumVals<'a> {
    pub values: &'a [&'a str],
}

#[derive(Serialize, Clone)]
pub struct UiOption<'a> {
    pub value: &'a str,
    pub label: &'a str,
}

#[derive(Serialize, Clone)]
pub struct FieldUi<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub section: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub widget: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_mode: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readonly: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<UiOption<'a>>>,
}

#[derive(Serialize, Clone)]
pub struct FieldRule<'a> {
    pub name: &'a str,
    pub r#type: &'a str,
    pub required: bool,
    pub nullable: bool,
    pub pii: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_len: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_len: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#enum: Option<EnumVals<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui: Option<FieldUi<'a>>,
}

#[derive(Serialize, Clone)]
pub struct Coercions<'a> {
    pub empty_string_is_null: bool,
    pub numeric_string_to_number: &'a [&'a str],
}

#[derive(Serialize, Clone)]
pub struct Rules {
    pub strip_control_chars: bool,
    pub max_body_bytes: u32,
}

#[derive(Serialize, Clone)]
pub struct ActionRule<'a> {
    pub model: &'a str,
    pub action: &'a str,
    pub version: &'a str,
    pub fields: Vec<FieldRule<'a>>,
    pub coercions: Coercions<'a>,
    pub rules: Rules,
}

pub fn common_rules() -> Rules {
    Rules {
        strip_control_chars: true,
        max_body_bytes: 32768,
    }
}

pub fn empty_coercions() -> Coercions<'static> {
    Coercions {
        empty_string_is_null: true,
        numeric_string_to_number: &[],
    }
}

pub fn stub_rule(model: &'static str, action: &'static str) -> ActionRule<'static> {
    ActionRule {
        model,
        action,
        version: "0",
        fields: Vec::new(),
        coercions: empty_coercions(),
        rules: common_rules(),
    }
}
