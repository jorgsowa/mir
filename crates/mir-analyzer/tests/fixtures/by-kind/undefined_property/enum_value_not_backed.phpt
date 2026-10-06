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
//                 ^^^^^ UndefinedProperty: Property Suit::$value does not exist
