===description===
Insufficient matches
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
UnhandledMatchCondition@9:15-12:9: Unhandled match condition: Suit::Spades
