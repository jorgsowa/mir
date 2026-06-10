===description===
Cant compare to suit twice
===file===
<?php
enum Suit {
    case Hearts;
    case Diamonds;
    case Clubs;
    case Spades;
}

function foo(Suit $s): void {
    if ($s === Suit::Clubs)  {
        if ($s === Suit::Clubs) {
            echo "bad";
        }
    }
}
===expect===
