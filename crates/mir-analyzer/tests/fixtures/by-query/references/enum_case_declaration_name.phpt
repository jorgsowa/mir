===description===
References from an enum case declaration name include its uses.
===cursor===
references include_declaration
===file===
<?php
enum Suit {
    case He<CURSOR>arts;
}
$s = Suit::Hearts;
===expect===
test.php@3:9-3:15
test.php@5:11-5:17
