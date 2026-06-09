===description===
Insufficient matches for cases
===file===
<?php
enum Suit {
    case Hearts;
    case Diamonds;
    case Clubs;
    case Spades;
}

foreach (Suit::cases() as $case) {
    echo match($case) {
        Suit::Hearts, Suit::Diamonds => "Red",
        Suit::Clubs => "Black",
    };
}
===expect===
UnhandledMatchCondition@10:10-13:11: Unhandled match condition: Suit::Spades
