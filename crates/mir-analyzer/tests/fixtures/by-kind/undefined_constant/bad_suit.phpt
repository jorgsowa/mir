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
UndefinedConstant@10:16-10:25: Constant Suit::Clu is not defined
