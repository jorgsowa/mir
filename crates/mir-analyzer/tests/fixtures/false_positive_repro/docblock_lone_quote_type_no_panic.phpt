===description===
A docblock type consisting of a single unmatched quote character (`'` or `"`)
must not panic. It satisfies both starts_with and ends_with for the string
literal case, so a naive length check on the byte slice would index out of
bounds. The analyzer must process the file without crashing, emitting at
most a parse-warning, never an ICE.
===config===
suppress=UnusedProperty,MissingPropertyType
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
