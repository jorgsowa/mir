===description===
A value outside the enum's case values still fails `value-of<Enum>`.
===file===
<?php
enum Suit: string {
    case Hearts = 'h';
    case Spades = 's';
}

/** @return value-of<Suit> */
function wrong(): string {
    return 'x';
//  ^^^^^^^^^^^ InvalidReturnType: Return type '"x"' is not compatible with declared 'value-of<Suit>'
}
