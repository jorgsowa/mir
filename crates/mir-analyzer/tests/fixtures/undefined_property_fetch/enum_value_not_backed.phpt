===description===
enumValueNotBacked
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
UndefinedPropertyFetch
===ignore===
TODO
