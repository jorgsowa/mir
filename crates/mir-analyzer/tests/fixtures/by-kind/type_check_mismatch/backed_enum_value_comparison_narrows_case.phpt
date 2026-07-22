===description===
`$enum->value === 'literal'` narrows $enum to the specific case whose
backing value equals the literal (and excludes that case on the false
branch) — sound because PHP requires distinct backing values across a
backed enum's cases, so the value uniquely identifies the case.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
enum Suit: string {
    case Hearts = 'H';
    case Spades = 'S';
    case Clubs = 'C';
}

function trueBranchNarrows(Suit $s): void {
    if ($s->value === 'H') {
        /** @mir-check $s is Suit::Hearts */
        $_ = 1;
    }
}

function falseBranchExcludes(Suit $s): void {
    if ($s->value === 'H') {
        return;
    }
    /** @mir-check $s is Suit::Spades|Suit::Clubs */
    $_ = 1;
}

function notIdenticalTrueBranchExcludes(Suit $s): void {
    if ($s->value !== 'H') {
        /** @mir-check $s is Suit::Spades|Suit::Clubs */
        $_ = 1;
    }
}

function notIdenticalFalseBranchNarrows(Suit $s): void {
    if ($s->value !== 'H') {
        return;
    }
    /** @mir-check $s is Suit::Hearts */
    $_ = 1;
}

function reversedOperandsNarrows(Suit $s): void {
    if ('H' === $s->value) {
        /** @mir-check $s is Suit::Hearts */
        $_ = 1;
    }
}
===expect===
