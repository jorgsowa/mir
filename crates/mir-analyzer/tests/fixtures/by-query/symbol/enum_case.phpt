===description===
An enum case fetch resolves to the case.
===cursor===
symbol
===file===
<?php
enum Suit {
    case Hearts;
    case Spades;
}
$s = Suit::Hea<CURSOR>rts;
===expect===
kind: class constant Suit::Hearts
type: Suit
