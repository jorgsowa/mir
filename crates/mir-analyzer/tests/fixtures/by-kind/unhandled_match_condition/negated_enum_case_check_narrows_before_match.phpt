===description===
FN: `$s === Status::Pending` narrowing the false branch only filtered atoms
when $s was already a union of per-case TLiteralEnumCase — a plain
`Status $s` parameter (a single generic enum atomic) never decomposed, so
the exclusion silently did nothing, leaving a later exhaustive `match`
wrongly flagged as missing the (already provably-excluded) Pending case.
===file===
<?php
enum Status { case Active; case Inactive; case Pending; }

function foo(Status $s): string {
    if ($s === Status::Pending) {
        return 'p';
    }
    return match ($s) {
        Status::Active => 'a',
        Status::Inactive => 'i',
    };
}
===expect===
