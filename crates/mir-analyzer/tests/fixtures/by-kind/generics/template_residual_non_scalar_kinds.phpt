===description===
`atomics_match_for_filter` now also recognizes array-shaped and
same-class named-object concrete alternatives, not just scalar kinds —
a `T|array` or `T|Foo` param no longer lets the array/object arg leak
into the template's binding instead of being filtered out.
===config===
suppress=UnusedParam,MissingThrowsDocblock,UnusedVariable
===file===
<?php
class Foo {}
class Bar {}

/**
 * @template T
 * @param T|array $value
 * @return T
 */
function unwrapArray($value) {
    throw new \Exception();
}

/** @param array<int, int>|Bar $x */
function test_array_alternative_filters_array_arg($x): void {
    $y = unwrapArray($x);
    /** @mir-check $y is Bar */
    $_ = $y;
}

/**
 * @template T
 * @param T|Foo $value
 * @return T
 */
function unwrapFoo($value) {
    throw new \Exception();
}

/** @param Foo|Bar $x */
function test_named_object_alternative_filters_same_class_arg($x): void {
    $y = unwrapFoo($x);
    /** @mir-check $y is Bar */
    $_ = $y;
}
===expect===
