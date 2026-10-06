===description===
A docblock type consisting of a single unmatched quote character (`'` or `"`)
is reported as an unterminated string literal instead of silently becoming
`mixed`. It previously crashed too: the lone quote satisfies both
starts_with and ends_with for the string-literal parse arm, so slicing it as
`s[1..s.len()-1]` panicked (a naive length check would still index out of
bounds even where it no longer parses as a literal).
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
    /** @var ' */
    public $bar;
//  ^^^^^^^^^^^ MissingPropertyType: Property Foo::$bar has no type annotation

    /** @var " */
    public $baz;
//  ^^^^^^^^^^^ MissingPropertyType: Property Foo::$baz has no type annotation
}
===expect===
InvalidDocblock@4:8-4:14: Invalid docblock: @var has an unterminated string literal in `'`
InvalidDocblock@8:8-8:14: Invalid docblock: @var has an unterminated string literal in `"`
