use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::Write;

use biome_analyze::{RuleFilter, RuleSource};
use biome_configuration::analyzer::{GroupPlainConfiguration, RuleGroupExt, SeverityOrGroup};
use biome_configuration::{Configuration, RulePlainConfiguration, Rules as RulesConfiguration};
use biome_diagnostics::Severity;
use biome_js_factory::make;
use biome_js_formatter::context::JsFormatOptions;
use biome_js_syntax::{
    AnyJsCallArgument, AnyJsExpression, AnyJsObjectMember, JsImport, JsSyntaxToken, T,
};
use biome_rowan::AstNode;

use crate::RuleRegistry;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum RuleSourceKind {
    Clippy,
    DenoLint,
    Eslint,
    EslintBarrelFiles,
    EslintGraphql,
    EslintImport,
    EslintImportAccess,
    EslintJest,
    EslintJsDoc,
    EslintJsxA11y,
    EslintMysticatea,
    EslintN,
    EslintNext,
    EslintNoSecrets,
    EslintPackageJson,
    EslintPackageJsonDependencies,
    EslintPerfectionist,
    EslintQwik,
    EslintReact,
    EslintReactHooks,
    EslintReactPreferFunctionComponent,
    EslintReactRefresh,
    EslintReactX,
    EslintReactXyz,
    EslintRegexp,
    EslintSolid,
    EslintSonarJs,
    EslintStylistic,
    EslintTypeScript,
    EslintUnicorn,
    EslintUnusedImports,
    EslintVitest,
    EslintVueJs,
    GraphqlSchemaLinter,
    Stylelint,
    EslintTurbo,
}

impl From<&RuleSource> for RuleSourceKind {
    fn from(value: &RuleSource) -> Self {
        match value {
            RuleSource::Clippy(_) => RuleSourceKind::Clippy,
            RuleSource::DenoLint(_) => RuleSourceKind::DenoLint,
            RuleSource::Eslint(_) => RuleSourceKind::Eslint,
            RuleSource::EslintBarrelFiles(_) => RuleSourceKind::EslintBarrelFiles,
            RuleSource::EslintGraphql(_) => RuleSourceKind::EslintGraphql,
            RuleSource::EslintImport(_) => RuleSourceKind::EslintImport,
            RuleSource::EslintImportAccess(_) => RuleSourceKind::EslintImportAccess,
            RuleSource::EslintJest(_) => RuleSourceKind::EslintJest,
            RuleSource::EslintJsDoc(_) => RuleSourceKind::EslintJsDoc,
            RuleSource::EslintJsxA11y(_) => RuleSourceKind::EslintJsxA11y,
            RuleSource::EslintMysticatea(_) => RuleSourceKind::EslintMysticatea,
            RuleSource::EslintN(_) => RuleSourceKind::EslintN,
            RuleSource::EslintNext(_) => RuleSourceKind::EslintNext,
            RuleSource::EslintNoSecrets(_) => RuleSourceKind::EslintNoSecrets,
            RuleSource::EslintPackageJson(_) => RuleSourceKind::EslintPackageJson,
            RuleSource::EslintPackageJsonDependencies(_) => {
                RuleSourceKind::EslintPackageJsonDependencies
            }
            RuleSource::EslintPerfectionist(_) => RuleSourceKind::EslintPerfectionist,
            RuleSource::EslintQwik(_) => RuleSourceKind::EslintQwik,
            RuleSource::EslintReact(_) => RuleSourceKind::EslintReact,
            RuleSource::EslintReactHooks(_) => RuleSourceKind::EslintReactHooks,
            RuleSource::EslintReactPreferFunctionComponent(_) => {
                RuleSourceKind::EslintReactPreferFunctionComponent
            }
            RuleSource::EslintReactRefresh(_) => RuleSourceKind::EslintReactRefresh,
            RuleSource::EslintReactX(_) => RuleSourceKind::EslintReactX,
            RuleSource::EslintReactXyz(_) => RuleSourceKind::EslintReactXyz,
            RuleSource::EslintRegexp(_) => RuleSourceKind::EslintRegexp,
            RuleSource::EslintSolid(_) => RuleSourceKind::EslintSolid,
            RuleSource::EslintSonarJs(_) => RuleSourceKind::EslintSonarJs,
            RuleSource::EslintStylistic(_) => RuleSourceKind::EslintStylistic,
            RuleSource::EslintTurbo(_) => RuleSourceKind::EslintTurbo,
            RuleSource::EslintTypeScript(_) => RuleSourceKind::EslintTypeScript,
            RuleSource::EslintUnicorn(_) => RuleSourceKind::EslintUnicorn,
            RuleSource::EslintUnusedImports(_) => RuleSourceKind::EslintUnusedImports,
            RuleSource::EslintVitest(_) => RuleSourceKind::EslintVitest,
            RuleSource::EslintVueJs(_) => RuleSourceKind::EslintVueJs,
            RuleSource::GraphqlSchemaLinter(_) => RuleSourceKind::GraphqlSchemaLinter,
            RuleSource::Stylelint(_) => RuleSourceKind::Stylelint,
        }
    }
}

impl RuleSourceKind {
    pub fn as_namespace(&self) -> Option<&'static str> {
        Some(match self {
            Self::EslintBarrelFiles => "barrel-files",
            Self::EslintGraphql => "@graphql-eslint",
            Self::EslintImport => "import",
            Self::EslintImportAccess => "import-access",
            Self::EslintJest => "jest",
            Self::EslintJsDoc => "jsdoc",
            Self::EslintJsxA11y => "jsx-a11y",
            Self::EslintMysticatea => "@mysticatea",
            Self::EslintN => "n",
            Self::EslintNext => "@next/next",
            Self::EslintNoSecrets => "no-secrets",
            Self::EslintPackageJson => "package-json",
            Self::EslintPackageJsonDependencies => "package-json-dependencies",
            Self::EslintPerfectionist => "perfectionist",
            Self::EslintQwik => "qwik",
            Self::EslintReact => "react",
            Self::EslintReactHooks => "react-hooks",
            Self::EslintReactPreferFunctionComponent => "react-prefer-function-component",
            Self::EslintReactRefresh => "react-refresh",
            Self::EslintReactX => "react-x",
            Self::EslintReactXyz => "@eslint-react",
            Self::EslintRegexp => "regexp",
            Self::EslintSolid => "solid",
            Self::EslintSonarJs => "sonarjs",
            Self::EslintStylistic => "@stylistic",
            Self::EslintTurbo => "turbo",
            Self::EslintTypeScript => "@typescript-eslint",
            Self::EslintUnicorn => "unicorn",
            Self::EslintUnusedImports => "unused-imports",
            Self::EslintVitest => "vitest",
            Self::EslintVueJs => "vue",
            _ => return None,
        })
    }

    fn to_ident(&self) -> Option<JsSyntaxToken> {
        Some(match self {
            Self::EslintTypeScript => make::ident("tseslint"),
            _ => return None, // TODO: Support other many sources
        })
    }

    fn to_import(&self, ident: JsSyntaxToken) -> Option<JsImport> {
        Some(match self {
            Self::EslintTypeScript => make::js_import(
                make::token_with_trailing_space(T![import]),
                make::js_import_default_clause(
                    make::js_default_import_specifier(make::js_identifier_binding(ident).into()),
                    make::token_decorated_with_space(T![from]),
                    make::js_module_source(make::js_string_literal("typescript-eslint")).into(),
                )
                .build()
                .into(),
            )
            .build(),
            _ => return None, // TODO: Support other many sources
        })
    }
}

fn group_config_to_severity(plain: &GroupPlainConfiguration) -> Option<Severity> {
    match plain {
        GroupPlainConfiguration::Error => Some(Severity::Error),
        GroupPlainConfiguration::Warn => Some(Severity::Warning),
        GroupPlainConfiguration::Info => Some(Severity::Information),
        _ => None,
    }
}

fn rule_config_to_severity(plain: RulePlainConfiguration) -> Option<Severity> {
    match plain {
        RulePlainConfiguration::Error => Some(Severity::Error),
        RulePlainConfiguration::Warn => Some(Severity::Warning),
        RulePlainConfiguration::Info => Some(Severity::Information),
        _ => None,
    }
}

fn severity_or_group_to_severity<G: RuleGroupExt>(
    severity_or_group: &SeverityOrGroup<G>,
    rule: &str,
) -> Option<Severity> {
    match severity_or_group {
        SeverityOrGroup::Plain(plain) => group_config_to_severity(plain),
        SeverityOrGroup::Group(group) => group
            .get_rule_configuration(rule)
            .and_then(|(plain, _)| rule_config_to_severity(plain)),
    }
}

fn get_configured_severity(
    config: &RulesConfiguration,
    group: &'static str,
    rule: &'static str,
) -> Option<Severity> {
    match group {
        "a11y" => config
            .a11y
            .as_ref()
            .and_then(|group| severity_or_group_to_severity(group, rule)),
        "complexity" => config
            .complexity
            .as_ref()
            .and_then(|group| severity_or_group_to_severity(group, rule)),
        "correctness" => config
            .correctness
            .as_ref()
            .and_then(|group| severity_or_group_to_severity(group, rule)),
        "nursery" => config
            .nursery
            .as_ref()
            .and_then(|group| severity_or_group_to_severity(group, rule)),
        "performance" => config
            .performance
            .as_ref()
            .and_then(|group| severity_or_group_to_severity(group, rule)),
        "security" => config
            .security
            .as_ref()
            .and_then(|group| severity_or_group_to_severity(group, rule)),
        "style" => config
            .style
            .as_ref()
            .and_then(|group| severity_or_group_to_severity(group, rule)),
        "suspicious" => config
            .performance
            .as_ref()
            .and_then(|group| severity_or_group_to_severity(group, rule)),
        _ => None,
    }
}

fn severity_to_eslint_level(severity: &Severity) -> &'static str {
    match severity {
        Severity::Error | Severity::Fatal => "error",
        Severity::Warning | Severity::Information | Severity::Hint => "warn",
    }
}

pub(crate) fn write_eslint_config(registry: &RuleRegistry, config: &Configuration) {
    let rules_config = config.get_linter_rules();
    let enabled_rules: BTreeSet<(&'static str, &'static str)> = rules_config
        .as_enabled_rules()
        .into_iter()
        .filter_map(|filter| match filter {
            RuleFilter::Group(_) => None,
            RuleFilter::Rule(group, rule) => Some((group, rule)),
        })
        .collect();

    let mut sources = BTreeSet::<RuleSourceKind>::new();
    let mut rules = BTreeMap::<String, Severity>::new();

    for (group, registry_rules) in &registry.groups {
        println!("{group}:");

        for (rule, metadata) in registry_rules {
            if !enabled_rules.contains(&(group, rule)) {
                continue;
            }

            let severity =
                get_configured_severity(&rules_config, group, rule).unwrap_or(metadata.severity);

            print!("  {rule}");

            let Some(rule_source) = metadata.sources.first() else {
                println!(" -> None");
                continue;
            };

            let source_kind = RuleSourceKind::from(&rule_source.source);
            let rule_name = rule_source.source.to_namespaced_rule_name();

            println!(" -> {} ({})", &rule_name, severity);

            sources.insert(source_kind);
            rules.insert(rule_name, severity);
        }
    }

    let mut imports = Vec::<JsImport>::new();
    let mut plugins = Vec::<AnyJsObjectMember>::new();

    for source in sources {
        // Built-in, nothing to do
        if source == RuleSourceKind::Eslint {
            continue;
        }

        let Some(ident) = source.to_ident() else {
            continue;
        };

        if let Some(import) = source.to_import(ident.clone())
            && let Some(namespace) = source.as_namespace()
        {
            imports.push(import);
            plugins.push(
                make::js_property_object_member(
                    make::js_literal_member_name(make::js_string_literal(namespace)).into(),
                    make::token_with_trailing_space(T![:]),
                    make::js_identifier_expression(make::js_reference_identifier(ident)).into(),
                )
                .into(),
            )
        }
    }

    // { "@typescript-eslint": tseslint, ... }
    let plugin_count = plugins.len();
    let plugins = make::js_object_expression(
        make::token(T!['{']),
        make::js_object_member_list(
            plugins,
            (0..plugin_count - 1).map(|_| make::token_with_trailing_space(T![,])),
        ),
        make::token(T!['}']),
    );

    // { "no-octal": "error", ... }
    let rules = make::js_object_expression(
        make::token(T!['{']),
        make::js_object_member_list(
            rules.iter().map(|(name, severity)| {
                make::js_property_object_member(
                    make::js_literal_member_name(make::js_string_literal(name.as_str())).into(),
                    make::token_with_trailing_space(T![:]),
                    AnyJsExpression::AnyJsLiteralExpression(
                        make::js_string_literal_expression(make::js_string_literal(
                            severity_to_eslint_level(severity),
                        ))
                        .into(),
                    ),
                )
                .into()
            }),
            (0..rules.len() - 1).map(|_| make::token_with_trailing_space(T![,])),
        ),
        make::token(T!['}']),
    );

    // import { defineConfig } from "eslint/config";
    imports.push(
        make::js_import(
            make::token_with_trailing_space(T![import]),
            make::js_import_named_clause(
                make::js_named_import_specifiers(
                    make::token_with_trailing_space(T!['{']),
                    make::js_named_import_specifier_list(
                        [make::js_shorthand_named_import_specifier(
                            make::js_identifier_binding(make::ident("defineConfig")).into(),
                        )
                        .build()
                        .into()],
                        [],
                    ),
                    make::token_with_leading_space(T!['}']),
                ),
                make::token_decorated_with_space(T![from]),
                make::js_module_source(make::js_string_literal("eslint/config")).into(),
            )
            .build()
            .into(),
        )
        .with_semicolon_token(make::token(T![;]))
        .build(),
    );

    // { plugins: ..., rules: ... }
    let config = make::js_object_expression(
        make::token(T!['{']),
        make::js_object_member_list(
            [
                make::js_property_object_member(
                    make::js_literal_member_name(make::ident("plugins")).into(),
                    make::token_with_trailing_space(T![:]),
                    plugins.into(),
                )
                .into(),
                make::js_property_object_member(
                    make::js_literal_member_name(make::ident("rules")).into(),
                    make::token_with_trailing_space(T![:]),
                    rules.into(),
                )
                .into(),
            ],
            [make::token_with_trailing_space(T![,])],
        ),
        make::token(T!['}']),
    );

    // defineConfig(...)
    let config = make::js_call_expression(
        make::js_identifier_expression(make::js_reference_identifier(make::ident("defineConfig")))
            .into(),
        make::js_call_arguments(
            make::token(T!['(']),
            make::js_call_argument_list([AnyJsCallArgument::AnyJsExpression(config.into())], []),
            make::token(T![')']),
        ),
    )
    .build();

    // export default ...
    let export = make::js_export(
        make::js_decorator_list([]),
        make::token_with_trailing_space(T![export]),
        make::js_export_default_expression_clause(
            make::token_with_trailing_space(T![default]),
            config.into(),
        )
        .build()
        .into(),
    );

    let module_items: Vec<_> = imports
        .into_iter()
        .map(|import| import.into())
        .chain(std::iter::once(export.into()))
        .collect();

    let root = make::js_module(
        make::js_directive_list([]),
        make::js_module_item_list(module_items),
        make::eof(),
    )
    .build();

    let options = JsFormatOptions::default();
    let formatted = biome_js_formatter::format_node(options, root.syntax()).unwrap();
    let printed = formatted.print().unwrap();

    File::create("eslint.config.mjs")
        .unwrap()
        .write_all(printed.as_code().as_bytes())
        .unwrap();
}
