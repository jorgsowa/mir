===description===
FALSE POSITIVE reproducer (J2). Narrowing a variable TO one specific enum
case via `===` and returning it must satisfy a declared return type of the
bare enum (nullable or not) — for both pure and backed enums. Covers the
InvalidReturnType path, distinct from the InvalidArgument path already
covered by the enum-case/bare-enum subtype fix's own fixtures.
Expected: no issue.
===config===
php_version=8.1
===file===
<?php
enum RoundingMode {
    case Unnecessary;
    case Up;
    case Down;
}

function narrowsToNullableReturn(RoundingMode $mode): ?RoundingMode {
    if ($mode === RoundingMode::Unnecessary) {
        return $mode;
    }
    return null;
}

function narrowsToBareReturn(RoundingMode $mode): RoundingMode {
    if ($mode === RoundingMode::Unnecessary) {
        return $mode;
    }
    return $mode;
}

enum Suit: string {
    case Hearts = 'H';
    case Spades = 'S';
    case Clubs = 'C';
}

function narrowsBackedEnumReturn(Suit $suit): ?Suit {
    if ($suit === Suit::Hearts) {
        return $suit;
    }
    return null;
}
===expect===
