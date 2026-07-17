===description===
An array-shape key consisting of a single unmatched quote character (e.g.
`array{': int}`) is reported as an unterminated string literal. Before the
fix, `parse_keyed_array` ran the key text through `strip_quotes`, which had
the same starts_with/ends_with-on-the-same-char bug as the string-literal
arm of `parse_type_string` and panicked on the same slice.
===config===
suppress=UnusedProperty
php_version=8.2
===file===
<?php

class Foo {
    /** @var array{': int} */
    public $bar;

    /** @var array{": int} */
    public $baz;
}
===expect===
InvalidDocblock@4:0-4:0: Invalid docblock: @var has an unterminated string literal in `array{': int}`
MissingPropertyType@5:4-5:15: Property Foo::$bar has no type annotation
InvalidDocblock@7:0-7:0: Invalid docblock: @var has an unterminated string literal in `array{": int}`
MissingPropertyType@8:4-8:15: Property Foo::$baz has no type annotation
