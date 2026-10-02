===description===
An enum case declaration name resolves to itself.
===cursor===
definition
===file===
<?php
enum Suit {
    case He<CURSOR>arts;
}
===expect===
test.php@3:4-3:16
