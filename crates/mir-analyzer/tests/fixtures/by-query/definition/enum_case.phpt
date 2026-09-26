===description===
Go-to-definition on an enum case lands on the `case` declaration.
===cursor===
definition
===file===
<?php
enum Suit {
    case Hearts;
    case Spades;
}
$s = Suit::Spa<CURSOR>des;
===expect===
test.php@4:4-4:16
