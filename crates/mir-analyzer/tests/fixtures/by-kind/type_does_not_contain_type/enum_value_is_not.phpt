===description===
Enum value is not: negative control for TypeDoesNotContainType — comparing a
backed enum's ->value against an unrelated string literal is still the SAME
structural type family (string vs string), so this diagnostic correctly never
fires here. `Suit::Hearts->value` now resolves to its own literal ("h") rather
than the bare backing type, so the comparison is also a genuine, provable
ImpossibleIdenticalComparison — a different (and correct) diagnostic, not the
one this fixture is a control for.
===file===
<?php
enum Suit: string {
    case Hearts = "h";
    case Diamonds = "d";
    case Clubs = "c";
    case Spades = "s";
}

if (Suit::Hearts->value === "a") {}
===expect===
ImpossibleIdenticalComparison@9:4-9:31: '===' between '"h"' and '"a"' is always false — these types can never be identical
