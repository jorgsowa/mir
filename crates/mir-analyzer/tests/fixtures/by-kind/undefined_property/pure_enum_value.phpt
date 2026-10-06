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
//               ^^^^^ UndefinedProperty: Property Direction::$value does not exist
}
