use mir_test_utils::fixture_test;

fixture_test!(unknown_function, "undefined_function/unknown_function.phpt");
fixture_test!(
    strlen_not_reported,
    "undefined_function/strlen_not_reported.phpt"
);
fixture_test!(
    array_map_not_reported,
    "undefined_function/array_map_not_reported.phpt"
);
fixture_test!(
    user_defined_function_not_reported,
    "undefined_function/user_defined_function_not_reported.phpt"
);
fixture_test!(
    global_namespace_unknown_function,
    "undefined_function/global_namespace_unknown_function.phpt"
);
fixture_test!(
    unpack_not_reported,
    "undefined_function/unpack_not_reported.phpt"
);
fixture_test!(
    suppressed_via_psalm_suppress,
    "undefined_function/suppressed_via_psalm_suppress.phpt"
);
fixture_test!(
    inside_method_body,
    "undefined_function/inside_method_body.phpt"
);
fixture_test!(
    multiple_call_sites,
    "undefined_function/multiple_call_sites.phpt"
);
