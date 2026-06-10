===description===
Enum value is not
===ignore===
TODO
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
