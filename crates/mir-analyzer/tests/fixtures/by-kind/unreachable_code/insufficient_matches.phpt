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
//             ^ +3:9 UnhandledMatchCondition: Unhandled match condition: Suit::Spades
            Suit::Hearts, Suit::Diamonds => "Red",
            Suit::Clubs => "Black",
        };
    }
}
