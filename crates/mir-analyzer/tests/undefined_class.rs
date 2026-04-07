use mir_test_utils::fixture_test;

fixture_test!(new_unknown_class, "undefined_class/new_unknown_class.phpt");
fixture_test!(
    stdclass_not_reported,
    "undefined_class/stdclass_not_reported.phpt"
);
fixture_test!(
    user_defined_class_not_reported,
    "undefined_class/user_defined_class_not_reported.phpt"
);
fixture_test!(
    unknown_param_type_hint,
    "undefined_class/unknown_param_type_hint.phpt"
);
fixture_test!(
    unknown_return_type_hint,
    "undefined_class/unknown_return_type_hint.phpt"
);
fixture_test!(
    extension_class_via_use_alias,
    "undefined_class/extension_class_via_use_alias.phpt"
);
fixture_test!(
    known_aliased_class_not_reported,
    "undefined_class/known_aliased_class_not_reported.phpt"
);
fixture_test!(
    instanceof_unknown_class,
    "undefined_class/instanceof_unknown_class.phpt"
);
fixture_test!(
    suppressed_via_psalm_suppress,
    "undefined_class/suppressed_via_psalm_suppress.phpt"
);
