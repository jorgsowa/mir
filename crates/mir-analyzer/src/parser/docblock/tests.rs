use super::*;
use mir_types::Atomic;

#[test]
fn parse_string() {
    let u = parse_type_string("string");
    assert_eq!(u.types.len(), 1);
    assert!(matches!(u.types[0], Atomic::TString));
}

#[test]
fn parse_nullable_string() {
    let u = parse_type_string("?string");
    assert!(u.is_nullable());
    assert!(u.contains(|t| matches!(t, Atomic::TString)));
}

#[test]
fn parse_union() {
    let u = parse_type_string("string|int|null");
    assert!(u.contains(|t| matches!(t, Atomic::TString)));
    assert!(u.contains(|t| matches!(t, Atomic::TInt)));
    assert!(u.is_nullable());
}

#[test]
fn parse_array_of_string() {
    let u = parse_type_string("array<string>");
    assert!(u.contains(|t| matches!(t, Atomic::TArray { .. })));
}

#[test]
fn parse_list_of_int() {
    let u = parse_type_string("list<int>");
    assert!(u.contains(|t| matches!(t, Atomic::TList { .. })));
}

#[test]
fn parse_named_class() {
    let u = parse_type_string("Foo\\Bar");
    assert!(u.contains(
        |t| matches!(t, Atomic::TNamedObject { fqcn, .. } if fqcn.as_ref() == "Foo\\Bar")
    ));
}

#[test]
fn parse_docblock_param_return() {
    let doc = r#"/**
         * @param string $name
         * @param int $age
         * @return bool
         */"#;
    let parsed = DocblockParser::parse(doc);
    assert_eq!(parsed.params.len(), 2);
    assert!(parsed.return_type.is_some());
    let ret = parsed.return_type.unwrap();
    assert!(ret.contains(|t| matches!(t, Atomic::TBool)));
}

#[test]
fn parse_template() {
    let doc = "/** @template T of object */";
    let parsed = DocblockParser::parse(doc);
    assert_eq!(parsed.templates.len(), 1);
    assert_eq!(parsed.templates[0].0, "T");
    assert!(parsed.templates[0].1.is_some());
    assert_eq!(parsed.templates[0].2, Variance::Invariant);
}

#[test]
fn parse_template_covariant() {
    let doc = "/** @template-covariant T */";
    let parsed = DocblockParser::parse(doc);
    assert_eq!(parsed.templates.len(), 1);
    assert_eq!(parsed.templates[0].0, "T");
    assert_eq!(parsed.templates[0].2, Variance::Covariant);
}

#[test]
fn parse_template_contravariant() {
    let doc = "/** @template-contravariant T */";
    let parsed = DocblockParser::parse(doc);
    assert_eq!(parsed.templates.len(), 1);
    assert_eq!(parsed.templates[0].0, "T");
    assert_eq!(parsed.templates[0].2, Variance::Contravariant);
}

#[test]
fn parse_template_single_line_does_not_over_read() {
    // E1: single-line docblock — the @template body runs to the closing `*/`,
    // so the parser used to take `T @param T $x @return T` as the template name.
    let doc = "/** @template T @param T $x @return T */";
    let parsed = DocblockParser::parse(doc);
    assert_eq!(parsed.templates.len(), 1);
    assert_eq!(parsed.templates[0].0, "T");
    assert!(parsed.templates[0].1.is_none(), "expected no bound");
}

#[test]
fn parse_template_multiline_with_bound_still_works() {
    // E1 regression guard: a normal multi-line `@template T of Base` keeps name + bound.
    let doc = r#"/**
         * @template T of Base
         */"#;
    let parsed = DocblockParser::parse(doc);
    assert_eq!(parsed.templates.len(), 1);
    assert_eq!(parsed.templates[0].0, "T");
    let bound = parsed.templates[0].1.as_ref().expect("expected a bound");
    assert!(bound
        .contains(|t| matches!(t, Atomic::TNamedObject { fqcn, .. } if fqcn.as_ref() == "Base")));
}

#[test]
fn parse_template_extends_alias() {
    // E2: `@template-extends` / `@phpstan-extends` route into `extends`.
    for tag in ["template-extends", "phpstan-extends"] {
        let doc = format!("/** @{tag} Base<User> */");
        let parsed = DocblockParser::parse(&doc);
        let extends = parsed
            .extends
            .unwrap_or_else(|| panic!("@{tag} should populate extends"));
        assert!(
            extends.contains(|t| matches!(
                t,
                Atomic::TNamedObject { fqcn, type_params }
                    if fqcn.as_ref() == "Base" && !type_params.is_empty()
            )),
            "@{tag} should produce a generic Base<User>"
        );
    }
}

#[test]
fn parse_template_implements_alias() {
    // E2: `@template-implements` / `@phpstan-implements` route into `implements`.
    for tag in ["template-implements", "phpstan-implements"] {
        let doc = format!("/** @{tag} Iter<User> */");
        let parsed = DocblockParser::parse(&doc);
        assert_eq!(
            parsed.implements.len(),
            1,
            "@{tag} should populate implements"
        );
        assert!(
            parsed.implements[0].contains(|t| matches!(
                t,
                Atomic::TNamedObject { fqcn, type_params }
                    if fqcn.as_ref() == "Iter" && !type_params.is_empty()
            )),
            "@{tag} should produce a generic Iter<User>"
        );
    }
}

#[test]
fn parse_deprecated() {
    let doc = "/** @deprecated use newMethod() instead */";
    let parsed = DocblockParser::parse(doc);
    assert!(parsed.is_deprecated);
    assert_eq!(
        parsed.deprecated.as_deref(),
        Some("use newMethod() instead")
    );
}

#[test]
fn parse_since_plain() {
    let parsed = DocblockParser::parse("/** @since 8.0 */");
    assert_eq!(parsed.since.as_deref(), Some("8.0"));
    assert_eq!(parsed.removed, None);
}

#[test]
fn parse_since_strips_trailing_description() {
    // phpstorm-stubs commonly writes `@since X.Y description text`.
    // Only the leading version token must reach the version parser.
    let parsed = DocblockParser::parse("/** @since 1.4.0 added \\$options argument */");
    assert_eq!(parsed.since.as_deref(), Some("1.4.0"));
}

#[test]
fn parse_removed_tag() {
    let parsed = DocblockParser::parse("/** @removed 8.0 use mb_convert_encoding */");
    assert_eq!(parsed.removed.as_deref(), Some("8.0"));
}

#[test]
fn parse_since_empty_body_is_none() {
    let parsed = DocblockParser::parse("/** @since */");
    assert_eq!(parsed.since, None);
}

#[test]
fn parse_description() {
    let doc = r#"/**
         * This is a description.
         * Spans two lines.
         * @param string $x
         */"#;
    let parsed = DocblockParser::parse(doc);
    assert!(parsed.description.contains("This is a description"));
    assert!(parsed.description.contains("Spans two lines"));
}

#[test]
fn parse_see_and_link() {
    let doc = "/** @see SomeClass\n * @link https://example.com */";
    let parsed = DocblockParser::parse(doc);
    assert_eq!(parsed.see.len(), 2);
    assert!(parsed.see.contains(&"SomeClass".to_string()));
    assert!(parsed.see.contains(&"https://example.com".to_string()));
}

#[test]
fn parse_mixin() {
    let doc = "/** @mixin SomeTrait */";
    let parsed = DocblockParser::parse(doc);
    assert_eq!(parsed.mixins, vec!["SomeTrait".to_string()]);
}

#[test]
fn parse_property_tags() {
    let doc = r#"/**
         * @property string $name
         * @property-read int $id
         * @property-write bool $active
         */"#;
    let parsed = DocblockParser::parse(doc);
    assert_eq!(parsed.properties.len(), 3);
    let name_prop = parsed.properties.iter().find(|p| p.name == "name").unwrap();
    assert_eq!(name_prop.type_hint, "string");
    assert!(!name_prop.read_only);
    assert!(!name_prop.write_only);
    let id_prop = parsed.properties.iter().find(|p| p.name == "id").unwrap();
    assert!(id_prop.read_only);
    let active_prop = parsed
        .properties
        .iter()
        .find(|p| p.name == "active")
        .unwrap();
    assert!(active_prop.write_only);
}

#[test]
fn parse_method_tag() {
    let doc = r#"/**
         * @method string getName()
         * @method static int create()
         */"#;
    let parsed = DocblockParser::parse(doc);
    assert_eq!(parsed.methods.len(), 2);
    let get_name = parsed.methods.iter().find(|m| m.name == "getName").unwrap();
    assert_eq!(get_name.return_type, "string");
    assert!(!get_name.is_static);
    let create = parsed.methods.iter().find(|m| m.name == "create").unwrap();
    assert!(create.is_static);
}

#[test]
fn parse_method_tag_description_with_parens() {
    // Carbon-style: description text contains "(using date interval)" after the
    // closing paren of the method signature. The old rfind(')') would capture
    // the description's closing paren and produce a phantom parameter.
    let doc = r#"/**
         * @method $this addDay() Add one day to the instance (using date interval).
         * @method $this subDays(int|float $value = 1) Sub days (the $value count passed in).
         */"#;
    let parsed = DocblockParser::parse(doc);
    let add_day = parsed.methods.iter().find(|m| m.name == "addDay").unwrap();
    assert_eq!(add_day.params.len(), 0, "addDay() must have zero params");
    let sub_days = parsed.methods.iter().find(|m| m.name == "subDays").unwrap();
    assert_eq!(sub_days.params.len(), 1);
    assert!(sub_days.params[0].is_optional);
}

#[test]
fn parse_type_alias_tag() {
    let doc = "/** @psalm-type MyAlias = string|int */";
    let parsed = DocblockParser::parse(doc);
    assert_eq!(parsed.type_aliases.len(), 1);
    assert_eq!(parsed.type_aliases[0].name, "MyAlias");
    assert_eq!(parsed.type_aliases[0].type_expr, "string|int");
}

#[test]
fn parse_import_type_no_as() {
    let doc = "/** @psalm-import-type UserId from UserRepository */";
    let parsed = DocblockParser::parse(doc);
    assert_eq!(parsed.import_types.len(), 1);
    assert_eq!(parsed.import_types[0].original, "UserId");
    assert_eq!(parsed.import_types[0].local, "UserId");
    assert_eq!(parsed.import_types[0].from_class, "UserRepository");
}

#[test]
fn parse_import_type_with_as() {
    let doc = "/** @psalm-import-type UserId as LocalId from UserRepository */";
    let parsed = DocblockParser::parse(doc);
    assert_eq!(parsed.import_types.len(), 1);
    assert_eq!(parsed.import_types[0].original, "UserId");
    assert_eq!(parsed.import_types[0].local, "LocalId");
    assert_eq!(parsed.import_types[0].from_class, "UserRepository");
}

#[test]
fn parse_require_extends() {
    let doc = "/** @psalm-require-extends Model */";
    let parsed = DocblockParser::parse(doc);
    assert_eq!(parsed.require_extends, vec!["Model".to_string()]);
}

#[test]
fn parse_require_implements() {
    let doc = "/** @psalm-require-implements Countable */";
    let parsed = DocblockParser::parse(doc);
    assert_eq!(parsed.require_implements, vec!["Countable".to_string()]);
}

#[test]
fn parse_intersection_two_parts() {
    let u = parse_type_string("Iterator&Countable");
    assert_eq!(u.types.len(), 1);
    assert!(matches!(u.types[0], Atomic::TIntersection { ref parts } if parts.len() == 2));
    if let Atomic::TIntersection { parts } = &u.types[0] {
        assert!(parts[0].contains(
            |t| matches!(t, Atomic::TNamedObject { fqcn, .. } if fqcn.as_ref() == "Iterator")
        ));
        assert!(parts[1].contains(
            |t| matches!(t, Atomic::TNamedObject { fqcn, .. } if fqcn.as_ref() == "Countable")
        ));
    }
}

#[test]
fn parse_intersection_three_parts() {
    let u = parse_type_string("Iterator&Countable&Stringable");
    assert_eq!(u.types.len(), 1);
    let Atomic::TIntersection { parts } = &u.types[0] else {
        panic!("expected TIntersection");
    };
    assert_eq!(parts.len(), 3);
    assert!(parts[0].contains(
        |t| matches!(t, Atomic::TNamedObject { fqcn, .. } if fqcn.as_ref() == "Iterator")
    ));
    assert!(parts[1].contains(
        |t| matches!(t, Atomic::TNamedObject { fqcn, .. } if fqcn.as_ref() == "Countable")
    ));
    assert!(parts[2].contains(
        |t| matches!(t, Atomic::TNamedObject { fqcn, .. } if fqcn.as_ref() == "Stringable")
    ));
}

#[test]
fn parse_intersection_in_union_with_null() {
    let u = parse_type_string("Iterator&Countable|null");
    assert!(u.is_nullable());
    let intersection = u
        .types
        .iter()
        .find_map(|t| {
            if let Atomic::TIntersection { parts } = t {
                Some(parts)
            } else {
                None
            }
        })
        .expect("expected TIntersection");
    assert_eq!(intersection.len(), 2);
    assert!(intersection[0].contains(
        |t| matches!(t, Atomic::TNamedObject { fqcn, .. } if fqcn.as_ref() == "Iterator")
    ));
    assert!(intersection[1].contains(
        |t| matches!(t, Atomic::TNamedObject { fqcn, .. } if fqcn.as_ref() == "Countable")
    ));
}

#[test]
fn parse_intersection_in_union_with_scalar() {
    let u = parse_type_string("Iterator&Countable|string");
    assert!(u.contains(|t| matches!(t, Atomic::TString)));
    let intersection = u
        .types
        .iter()
        .find_map(|t| {
            if let Atomic::TIntersection { parts } = t {
                Some(parts)
            } else {
                None
            }
        })
        .expect("expected TIntersection");
    assert!(intersection[0].contains(
        |t| matches!(t, Atomic::TNamedObject { fqcn, .. } if fqcn.as_ref() == "Iterator")
    ));
    assert!(intersection[1].contains(
        |t| matches!(t, Atomic::TNamedObject { fqcn, .. } if fqcn.as_ref() == "Countable")
    ));
}

#[test]
fn validate_unclosed_generic_return() {
    let parsed = DocblockParser::parse("/** @return array< */");
    assert_eq!(parsed.invalid_annotations.len(), 1);
    assert!(
        parsed.invalid_annotations[0].contains("unclosed generic"),
        "got: {}",
        parsed.invalid_annotations[0]
    );
}

#[test]
fn parse_empty_generic_array_graceful() {
    let u = parse_type_string("array<>");
    assert!(u.contains(|t| matches!(t, Atomic::TArray { .. })));
}

#[test]
fn parse_empty_generic_iterable_graceful() {
    let u = parse_type_string("iterable<>");
    assert!(u.contains(|t| matches!(t, Atomic::TArray { .. })));
}

#[test]
fn parse_empty_generic_non_empty_array_graceful() {
    let u = parse_type_string("non-empty-array<>");
    assert!(u.contains(|t| matches!(t, Atomic::TNonEmptyArray { .. })));
}

#[test]
fn validate_variable_in_type_position_param() {
    let parsed = DocblockParser::parse("/** @param Foo|$invalid $x */");
    assert_eq!(parsed.invalid_annotations.len(), 1);
    assert!(
        parsed.invalid_annotations[0].contains("$invalid"),
        "got: {}",
        parsed.invalid_annotations[0]
    );
}

#[test]
fn validate_this_is_valid_in_type_position() {
    let parsed = DocblockParser::parse("/** @return $this */");
    assert!(
        parsed.invalid_annotations.is_empty(),
        "unexpected error: {:?}",
        parsed.invalid_annotations
    );
}

#[test]
fn validate_unclosed_generic_var() {
    let parsed = DocblockParser::parse("/** @var array<string */");
    assert_eq!(parsed.invalid_annotations.len(), 1);
    assert!(parsed.invalid_annotations[0].contains("@var"));
}

#[test]
fn validate_variable_in_template_bound() {
    let parsed = DocblockParser::parse("/** @template T of $invalid */");
    assert_eq!(parsed.invalid_annotations.len(), 1);
    assert!(parsed.invalid_annotations[0].contains("$invalid"));
}
