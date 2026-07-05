===description===
Comparing an already-narrowed enum-case variable against the same case again
is now recognized as always true, once EnumName::CaseName (real syntax,
ClassConstAccess) is recognized by narrowing instead of only the unreachable
StaticPropertyAccess shape.
===file===
<?php
enum Suit {
    case Hearts;
    case Diamonds;
    case Clubs;
    case Spades;
}

function foo(Suit $s): void {
    if ($s === Suit::Clubs)  {
        if ($s === Suit::Clubs) {
            echo "bad";
        }
    }
}
===expect===
RedundantCondition@11:12-11:30: Condition is always true/false for type 'bool'
