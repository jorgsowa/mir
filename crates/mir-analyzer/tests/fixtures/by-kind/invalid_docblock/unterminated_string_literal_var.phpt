===description===
A docblock type consisting of a single unmatched quote character (`'` or `"`)
is reported as an unterminated string literal instead of silently becoming
`mixed`. It previously crashed too: the lone quote satisfies both
starts_with and ends_with for the string-literal parse arm, so slicing it as
`s[1..s.len()-1]` panicked (a naive length check would still index out of
bounds even where it no longer parses as a literal).
===config===
suppress=UnusedProperty
php_version=8.2
===file===
<?php

class Foo {
    /** @var ' */
    public $bar;

    /** @var " */
    public $baz;
}
===expect===
InvalidDocblock@4:0-4:0: Invalid docblock: @var has an unterminated string literal in `'`
MissingPropertyType@5:4-5:15: Property Foo::$bar has no type annotation
InvalidDocblock@7:0-7:0: Invalid docblock: @var has an unterminated string literal in `"`
MissingPropertyType@8:4-8:15: Property Foo::$baz has no type annotation
