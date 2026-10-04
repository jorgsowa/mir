===description===
`value-of<Enum>` expands to the backed enum's case values, so `->value` and matching literals satisfy it.
===file===
<?php
enum Suit: string {
    case Hearts = 'h';
    case Spades = 's';
}

enum Level: int {
    case Low = 1;
    case High = 2;
}

/** @return value-of<Suit>|null */
function nullableValue(Suit $s): ?string {
    return $s->value;
}

/** @return value-of<Suit> */
function literal(): string {
    return 'h';
}

/** @return value-of<Level> */
function intValue(Level $l): int {
    return $l->value;
}
===expect===
