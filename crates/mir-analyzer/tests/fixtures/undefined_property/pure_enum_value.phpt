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
UndefinedProperty: Property Direction::$value does not exist
