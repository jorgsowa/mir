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
}
===expect===
UndefinedConstant@10:15-10:24: Constant Suit::Clu is not defined
