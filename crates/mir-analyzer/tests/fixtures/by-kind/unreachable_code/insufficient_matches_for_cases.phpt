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
//       ^ +3:5 UnhandledMatchCondition: Unhandled match condition: Suit::Spades
        Suit::Hearts, Suit::Diamonds => "Red",
        Suit::Clubs => "Black",
    };
}
