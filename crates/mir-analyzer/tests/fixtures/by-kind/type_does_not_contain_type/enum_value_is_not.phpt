===description===
Enum value is not
===file===
<?php
enum Suit: string {
    case Hearts = "h";
    case Diamonds = "d";
    case Clubs = "c";
    case Spades = "s";
}

if (Suit::Hearts->value === "a") {}
===expect===
TypeDoesNotContainType
===ignore===
TODO
