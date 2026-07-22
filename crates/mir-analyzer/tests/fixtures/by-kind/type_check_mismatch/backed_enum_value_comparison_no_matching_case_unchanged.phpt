===description===
A literal that doesn't match any case's backing value (an impossible
comparison) is left alone rather than narrowed to something incorrect —
the type stays the full enum union in both branches.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
enum Suit: string {
    case Hearts = 'H';
    case Spades = 'S';
}

function noMatch(Suit $s): void {
    if ($s->value === 'X') {
        /** @mir-check $s is Suit */
        $_ = 1;
    } else {
        /** @mir-check $s is Suit */
        $_ = 1;
    }
}
===expect===
