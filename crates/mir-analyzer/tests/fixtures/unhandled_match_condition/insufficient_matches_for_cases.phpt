===description===
insufficientMatchesForCases
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
UnhandledMatchCondition
===ignore===
TODO
