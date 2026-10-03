===description===
An array-shape key consisting of a single unmatched quote character (e.g.
`array{': int}`) is reported as an unterminated string literal. Before the
fix, `parse_keyed_array` ran the key text through `strip_quotes`, which had
the same starts_with/ends_with-on-the-same-char bug as the string-literal
arm of `parse_type_string` and panicked on the same slice.
===config===
<mir>
  <issueHandlers>
    <UnusedProperty errorLevel="suppress"/>
  </issueHandlers>
  <phpVersion>8.2</phpVersion>
</mir>
===file===
<?php

class Foo {
    /** @var array{': int} */
    public $bar;
//  ^^^^^^^^^^^ MissingPropertyType: Property Foo::$bar has no type annotation

    /** @var array{": int} */
    public $baz;
//  ^^^^^^^^^^^ MissingPropertyType: Property Foo::$baz has no type annotation
}
===expect===
InvalidDocblock@4:0-4:0: Invalid docblock: @var has an unterminated string literal in `array{': int}`
InvalidDocblock@8:0-8:0: Invalid docblock: @var has an unterminated string literal in `array{": int}`
