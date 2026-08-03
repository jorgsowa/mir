===description===
P24 control: a chain of ANDed negated `instanceof` checks must only exclude
the classes it actually names — a `Bad1` that isn't excluded by either
conjunct must still be flagged as a real UndefinedMethod, proving the De
Morgan narrowing fix (see the false_positive_repro P24 fixtures) doesn't
over-narrow.
===file===
<?php

final class Good1 { public function file(): string { return 'x'; } }
final class Bad1 {}
final class Bad2 {}

function reasonLocation(Good1|Bad1|Bad2 $reason): string
{
    if (!$reason instanceof Good1 && !$reason instanceof Bad1) {
        return '';
    }
    return $reason->file();
}
===expect===
MixedReturnStatement@12:4-12:27: Cannot return a mixed type from function with declared return type 'string'
UndefinedMethod@12:11-12:26: Method Bad1::file() does not exist
