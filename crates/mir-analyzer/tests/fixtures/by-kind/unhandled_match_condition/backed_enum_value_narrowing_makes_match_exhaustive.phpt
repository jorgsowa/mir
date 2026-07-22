===description===
A `match` over a backed enum is seen as exhaustive when an earlier
`->value ===` comparison (and early return) has already narrowed out one
of the cases — without `->value` narrowing, this match looks like it's
missing the `Hearts` arm.
===config===
suppress=UnusedParam
===file===
<?php
enum Suit: string {
    case Hearts = 'H';
    case Spades = 'S';
}

function describe(Suit $s): string {
    if ($s->value === 'H') {
        return 'hearts';
    }

    return match ($s) {
        Suit::Spades => 'spades',
    };
}
===expect===
