===description===
An unterminated string literal as one member of a union (`'a'|'`) must
still be caught — the check runs on the whole union body before it gets
split into members, since splitting on `|` is itself quote-aware and would
otherwise swallow the trailing member's unmatched quote silently.
===config===
suppress=UnusedProperty
===file===
<?php

class Foo {
    /** @var 'a'|' */
    public $bar;
}
===expect===
InvalidDocblock@4:0-4:0: Invalid docblock: @var has an unterminated string literal in `'a'|'`
MissingPropertyType@5:4-5:15: Property Foo::$bar has no type annotation
