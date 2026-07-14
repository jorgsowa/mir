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
fn parse_bare_array_keys_on_array_key_not_mixed() {
    // Regression guard: a bare `array` used to build its key as a literal
    // `mixed`, which both misrepresented the true PHP array-key domain
    // (`int|string`) and defeated the `array<mixed, mixed>` -> `array`
    // display collapse for the common no-generics case.
    let u = parse_type_string("array");
    assert!(u.contains(|t| matches!(t, Atomic::TArray { key, .. } if key.is_array_key())));
}

#[test]
fn parse_bare_array_displays_as_bare_array() {
    let u = parse_type_string("array");
    assert_eq!(format!("{u}"), "array");
}

#[test]
fn parse_bare_list_displays_as_bare_list() {
    let u = parse_type_string("list");
    assert_eq!(format!("{u}"), "list");
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
            .first()
            .cloned()
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

// ---------------------------------------------------------------------------
// Float-literal types
// ---------------------------------------------------------------------------

#[test]
fn parse_float_literal_positive() {
    let u = parse_type_string("3.14");
    assert_eq!(u.types.len(), 1);
    assert!(matches!(u.types[0], Atomic::TLiteralFloat(..)));
    assert_eq!(format!("{u}"), "3.14");
}

#[test]
fn parse_float_literal_negative() {
    let u = parse_type_string("-0.5");
    assert!(matches!(u.types[0], Atomic::TLiteralFloat(..)));
    assert_eq!(format!("{u}"), "-0.5");
}

#[test]
fn plain_integer_is_not_float_literal() {
    let u = parse_type_string("42");
    assert!(matches!(u.types[0], Atomic::TLiteralInt(42)));
}

#[test]
fn dotted_non_number_is_not_float_literal() {
    // A malformed token with a dot must not be mistaken for a float.
    let u = parse_type_string("1.2.3");
    assert!(!matches!(u.types[0], Atomic::TLiteralFloat(..)));
}

// ---------------------------------------------------------------------------
// Psalm string refinements / int-mask
// ---------------------------------------------------------------------------

#[test]
fn parse_truthy_and_non_falsy_string() {
    assert!(matches!(
        parse_type_string("truthy-string").types[0],
        Atomic::TNonEmptyString
    ));
    assert!(matches!(
        parse_type_string("non-falsy-string").types[0],
        Atomic::TNonEmptyString
    ));
}

#[test]
fn parse_case_constrained_string() {
    assert!(matches!(
        parse_type_string("lowercase-string").types[0],
        Atomic::TString
    ));
    assert!(matches!(
        parse_type_string("uppercase-string").types[0],
        Atomic::TString
    ));
}

#[test]
fn parse_int_mask_expands_to_literal_union() {
    // int-mask<1, 2, 4> → all OR-combinations: 0,1,2,3,4,5,6,7
    let u = parse_type_string("int-mask<1, 2, 4>");
    assert_eq!(
        u.types.len(),
        8,
        "expected 2^3=8 members, got {}",
        u.types.len()
    );
    for n in 0i64..8 {
        assert!(
            u.contains(|t| matches!(t, Atomic::TLiteralInt(m) if *m == n)),
            "missing literal {n} in int-mask<1,2,4> union: {u}"
        );
    }
}

#[test]
fn parse_int_mask_single_member() {
    // int-mask<4> → {0, 4}
    let u = parse_type_string("int-mask<4>");
    assert_eq!(u.types.len(), 2);
    assert!(u.contains(|t| matches!(t, Atomic::TLiteralInt(0))));
    assert!(u.contains(|t| matches!(t, Atomic::TLiteralInt(4))));
}

#[test]
fn parse_int_mask_with_zero_member_deduplicates() {
    // int-mask<0, 1> → {0, 1} (0 is already included as the empty subset)
    let u = parse_type_string("int-mask<0, 1>");
    assert_eq!(u.types.len(), 2);
    assert!(u.contains(|t| matches!(t, Atomic::TLiteralInt(0))));
    assert!(u.contains(|t| matches!(t, Atomic::TLiteralInt(1))));
}

#[test]
fn parse_int_mask_too_many_members_falls_back() {
    // > 8 members → fall back to int
    let u = parse_type_string("int-mask<1, 2, 4, 8, 16, 32, 64, 128, 256>");
    assert_eq!(u.types.len(), 1);
    assert!(matches!(u.types[0], Atomic::TInt));
}

#[test]
fn parse_int_mask_of_always_falls_back() {
    // int-mask-of cannot resolve class constants at parse time → always int
    let u = parse_type_string("int-mask-of<Foo::*>");
    assert_eq!(u.types.len(), 1);
    assert!(matches!(u.types[0], Atomic::TInt));
}

#[test]
fn parse_int_mask_class_constants_fall_back() {
    // Members that are not integer literals → fall back to int
    let u = parse_type_string("int-mask<SORT_ASC, SORT_DESC>");
    assert_eq!(u.types.len(), 1);
    assert!(matches!(u.types[0], Atomic::TInt));
}

// ---------------------------------------------------------------------------
// key-of / value-of evaluation
// ---------------------------------------------------------------------------

#[test]
fn key_of_array_is_key_type() {
    let u = parse_type_string("key-of<array<int, string>>");
    assert_eq!(u.types.len(), 1);
    assert!(matches!(u.types[0], Atomic::TInt));
}

#[test]
fn key_of_list_is_int() {
    let u = parse_type_string("key-of<list<string>>");
    assert!(matches!(u.types[0], Atomic::TInt));
}

#[test]
fn key_of_keyed_array_is_literal_keys() {
    let u = parse_type_string("key-of<array{a: int, b: int}>");
    assert!(u.contains(|t| matches!(t, Atomic::TLiteralString(s) if s.as_ref() == "a")));
    assert!(u.contains(|t| matches!(t, Atomic::TLiteralString(s) if s.as_ref() == "b")));
}

#[test]
fn value_of_array_is_value_type() {
    let u = parse_type_string("value-of<array<int, string>>");
    assert!(matches!(u.types[0], Atomic::TString));
}

#[test]
fn value_of_keyed_array_is_value_union() {
    let u = parse_type_string("value-of<array{a: \"foo\", b: \"bar\"}>");
    assert!(u.contains(|t| matches!(t, Atomic::TLiteralString(s) if s.as_ref() == "foo")));
    assert!(u.contains(|t| matches!(t, Atomic::TLiteralString(s) if s.as_ref() == "bar")));
}

#[test]
fn value_of_union_of_lists_and_shapes() {
    let u = parse_type_string("value-of<list<0|1|2>|array{0: 3, 1: 4}>");
    for n in [0, 1, 2, 3, 4] {
        assert!(
            u.contains(|t| matches!(t, Atomic::TLiteralInt(m) if *m == n)),
            "missing literal {n} in {u}"
        );
    }
}

#[test]
fn key_of_unresolvable_falls_back_to_mixed() {
    // key-of over a named class / template cannot be resolved statically.
    let u = parse_type_string("key-of<\\SplStack>");
    assert!(u.is_mixed());
}

// ---------------------------------------------------------------------------
// parse_param_line: first-match / depth-tracking
// ---------------------------------------------------------------------------

#[test]
fn parse_param_line_description_with_dollar_var_ignored() {
    // Description text that contains a $var reference must not bleed into
    // the type or replace the param name with the var from the description.
    let doc = "/** @param bool $flag Whether the $option should be enabled */";
    let parsed = DocblockParser::parse(doc);
    assert_eq!(parsed.params.len(), 1, "should register exactly one param");
    let (name, ty) = &parsed.params[0];
    assert_eq!(name, "flag", "param name must be 'flag', not 'option'");
    assert!(
        ty.contains(|t| matches!(t, Atomic::TBool)),
        "param type must be bool, got {ty}"
    );
}

#[test]
fn parse_param_line_callable_dollar_inside_parens_not_the_name() {
    // $a inside callable(int $a) is at depth > 0 and must not be chosen as
    // the parameter name; $callback at depth 0 is the correct match.
    let doc = "/** @param callable(int $a): void $callback The callback to invoke */";
    let parsed = DocblockParser::parse(doc);
    assert_eq!(parsed.params.len(), 1);
    let (name, _ty) = &parsed.params[0];
    assert_eq!(name, "callback", "param name must be 'callback', not 'a'");
}

#[test]
fn parse_param_line_byref_param_name_correct() {
    // &$out — the leading & must be stripped; the param name is 'out'.
    let doc = "/** @param string &$out The output buffer */";
    let parsed = DocblockParser::parse(doc);
    assert_eq!(parsed.params.len(), 1);
    let (name, ty) = &parsed.params[0];
    assert_eq!(name, "out");
    assert!(ty.contains(|t| matches!(t, Atomic::TString)));
}
