===description===
FALSE POSITIVE reproducer. Valid PHP: `Color::{$name}` is dynamic enum-case access on a defined enum, not a bare constant.
Expected: no issue.
===config===
php_version=8.4
suppress=MixedReturnStatement
===file===
<?php
enum Color: string {
    case Red = 'red';
    case Blue = 'blue';
}
function pick(string $name): Color {
    return Color::{$name};
}
===expect===
