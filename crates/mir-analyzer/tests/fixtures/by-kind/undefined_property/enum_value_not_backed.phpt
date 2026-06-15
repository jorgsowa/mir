===description===
Enum value not backed
===file===
<?php
enum Suit {
    case Hearts;
    case Diamonds;
    case Clubs;
    case Spades;
}

echo Suit::Hearts->value;
===expect===
UndefinedProperty@9:19-9:24: Property Suit::$value does not exist
