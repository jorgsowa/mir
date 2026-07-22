===description===
Regression guard: a plain (non-enum) object with its own `value` property
compared against a string literal must not be mistaken for the backed-enum
`->value` idiom — the receiver's type is left completely alone.
===config===
suppress=UnusedVariable,UnusedParam,MissingPropertyType
===file===
<?php
final class Money {
    public $value = 'USD';
}

function notAnEnum(Money $m): void {
    if ($m->value === 'USD') {
        /** @mir-check $m is Money */
        $_ = 1;
    }
}
===expect===
