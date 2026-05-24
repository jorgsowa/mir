===description===
insufficientMatches
===file===
<?php
enum Suit {
    case Hearts;
    case Diamonds;
    case Clubs;
    case Spades;

    public function color(): string {
        return match($this) {
            Suit::Hearts, Suit::Diamonds => "Red",
            Suit::Clubs => "Black",
        };
    }
}
===expect===
UnhandledMatchCondition
===ignore===
TODO
