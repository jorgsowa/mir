===description===
An array-shape key consisting of a single unmatched quote character (e.g.
`array{': int}`) must not panic. `parse_keyed_array` runs the key text
through `strip_quotes`, which had the same starts_with/ends_with-on-the-
same-char bug as the string-literal arm of `parse_type_string` (both
conditions are trivially true for a one-character string, so slicing
`s[1..s.len()-1]` panicked). The analyzer must process the file without
crashing, emitting at most a parse-warning, never an ICE.
===config===
suppress=UnusedProperty,MissingPropertyType,InvalidDocblock
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
