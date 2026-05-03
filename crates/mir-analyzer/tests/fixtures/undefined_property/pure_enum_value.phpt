===description===
pure enum value
===file===
<?php
enum Direction {
    case North;
    case South;
}
function test(Direction $dir): mixed {
    return $dir->value;
}
===expect===
UndefinedProperty@7:17: Property Direction::$value does not exist
===ignore===
TODO
