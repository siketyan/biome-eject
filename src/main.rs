mod eslint;

use std::collections::BTreeMap;
use std::fs::File;

use biome_analyze::{Queryable, RegistryVisitor, Rule, RuleGroup, RuleMetadata};
use biome_configuration::Configuration;
use biome_js_analyze::visit_registry;
use biome_js_syntax::JsLanguage;

use crate::eslint::write_eslint_config;

type Rules = BTreeMap<&'static str, RuleMetadata>;
type Groups = BTreeMap<&'static str, Rules>;

#[derive(Default)]
struct RuleRegistry {
    groups: Groups,
}

impl RegistryVisitor<JsLanguage> for RuleRegistry {
    fn record_rule<R>(&mut self)
    where
        R: Rule<Query: Queryable<Language = JsLanguage, Output: Clone>> + 'static,
    {
        let group = R::Group::NAME;
        let metadata = R::METADATA;

        self.groups
            .entry(group)
            .or_insert_with(Default::default)
            .insert(metadata.name, metadata);
    }
}

fn main() {
    let config = File::open("biome.json")
        .or_else(|_| File::open("biome.jsonc"))
        .unwrap();

    let config: Configuration = serde_json::from_reader(&config).unwrap();

    let mut registry = RuleRegistry::default();

    visit_registry(&mut registry);

    if config.is_linter_enabled() {
        write_eslint_config(&registry, &config);

        // TODO: Install plugins automatically?
    }

    // TODO: Support Prettier

    // TODO: Support overrides

    // TODO: Uninstall Biome?
}
