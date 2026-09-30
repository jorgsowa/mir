===description===
Bad suit
===file===
<?php
enum Suit {
    case Hearts;
    case Diamonds;
    case Clubs;
    case Spades;
}

function foo(Suit $s): void {
    if ($s === Suit::Clu) {}
//             ^^^^^^^^^ UndefinedConstant: Constant Suit::Clu is not defined
}
===expect===
