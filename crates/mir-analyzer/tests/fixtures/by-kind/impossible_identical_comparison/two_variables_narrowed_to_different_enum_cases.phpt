===description===
Two variables independently narrowed to different cases of the same enum via
real `EnumName::CaseName` (===) comparisons make a later `===` between them
statically impossible.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
enum Suit {
    case Hearts;
    case Spades;
}

function f(Suit $a, Suit $b): bool {
    if ($a === Suit::Hearts && $b === Suit::Spades) {
        return $a === $b;
    }
    return false;
}
===expect===
ImpossibleIdenticalComparison@9:15-9:24: '===' between 'Suit::Hearts' and 'Suit::Spades' is always false — these types can never be identical
