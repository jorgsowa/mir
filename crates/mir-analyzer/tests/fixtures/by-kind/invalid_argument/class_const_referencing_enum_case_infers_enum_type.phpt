===description===
`const DEFAULT = Suit::Hearts;` (no docblock/native hint) infers the enum
type from the referenced case, instead of falling back to `mixed` — a value
of the wrong type is still caught.
===config===
suppress=UnusedParam
===file===
<?php
enum Suit {
    case Hearts;
    case Spades;
}
class Card {
    const DEFAULT = Suit::Hearts;
}
function takesString(string $s): void {}
function f(): void {
    takesString(Card::DEFAULT);
}
===expect===
InvalidArgument@11:16-11:29: Argument $s of takesString() expects 'string', got 'Suit'
