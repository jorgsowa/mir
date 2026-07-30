===description===
Negative control for the enum-`->value`-match-exhaustiveness fix: a
genuinely missing case must still be flagged, naming exactly the
uncovered case.
===config===
suppress=UnusedParam
===file===
<?php
enum Kind: string {
    case Foo = 'foo';
    case Bar = 'bar';
    case Baz = 'baz';
}
function h(Kind $type): bool {
    return match ($type->value) {
        Kind::Foo->value => true,
        Kind::Bar->value => false,
    };
}
===expect===
UnhandledMatchCondition@8:11-11:5: Unhandled match condition: Kind::Baz->value
