===description===
`value-of<Enum>` parameters accept the enum's case values and reject others.
===file===
<?php
enum Suit: string {
    case Hearts = 'h';
    case Spades = 's';
}

/** @param value-of<Suit> $v */
function take(string $v): void {
    echo $v;
}

function run(Suit $s): void {
    take('h');
    take($s->value);
//       ^^^^^^^^^ ArgumentTypeCoercion: Argument $v of take() expects '"h"|"s"', got 'string' — coercion may fail at runtime
    take('x');
//       ^^^ InvalidArgument: Argument $v of take() expects '"h"|"s"', got '"x"'
}
