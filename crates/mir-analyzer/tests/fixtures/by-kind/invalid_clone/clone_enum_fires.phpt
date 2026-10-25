===description===
Enum cases cannot be cloned, whether the type is a known case or the enum itself.
===file===
<?php
enum Suit {
    case Hearts;
}

function copy(Suit $suit): Suit {
    return clone $suit;
//         ^^^^^^^^^^^ InvalidClone: Trying to clone an uncloneable object of class Suit
}

copy(clone Suit::Hearts);
//   ^^^^^^^^^^^^^^^^^^ InvalidClone: Trying to clone an uncloneable object of class Suit
