===description===
Enum case references are listed per case, not for sibling cases.
===cursor===
references
===file===
<?php
enum Suit {
    case Hearts;
    case Spades;
}
$a = Suit::Hea<CURSOR>rts;
$b = Suit::Spades;
$c = Suit::Hearts;
===expect===
test.php@6:11-6:17
test.php@8:11-8:17
