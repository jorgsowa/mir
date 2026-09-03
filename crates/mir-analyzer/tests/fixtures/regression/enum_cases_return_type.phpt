===description===
Enum ::cases() returns list<EnumType> for both pure and backed enums.
Expected: no issue.
===config===
php_version=8.1
suppress=UnusedVariable
===file===
<?php
enum Color: string {
    case Red = 'r';
    case Green = 'g';
}

enum Direction {
    case North;
    case South;
}

$colors = Color::cases();
/** @mir-check $colors is list<Color> */
foreach ($colors as $c) {
    echo $c->value;
}

$dirs = Direction::cases();
/** @mir-check $dirs is list<Direction> */
foreach ($dirs as $d) {
    echo $d->name;
}
===expect===
