===description===
A `match` on a backed enum's `->value` must be recognized as exhaustive
when every arm is `EnumName::Case->value` and all cases are covered —
`match($enum)` directly on the enum object already narrows correctly;
`->value` specifically lost that per-case literal, widening to the plain
backing scalar type on both the subject and (via `Kind::Foo->value`) the
arms, so no finite arm set could ever "prove" coverage.
===config===
suppress=UnusedParam
===file===
<?php
enum Kind: string {
    case Foo = 'foo';
    case Bar = 'bar';
}
function h(Kind $type): bool {
    return match ($type->value) {
        Kind::Foo->value => true,
        Kind::Bar->value => false,
    };
}
===expect===
