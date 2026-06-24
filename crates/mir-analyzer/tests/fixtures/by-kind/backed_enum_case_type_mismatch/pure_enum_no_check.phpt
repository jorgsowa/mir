===description===
P6(a): Pure enums (no backing type) are not validated — their cases have no values.
===file===
<?php
enum Suit {
    case Hearts;
    case Diamonds;
    case Clubs;
    case Spades;
}
===expect===
