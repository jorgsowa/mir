===description===
FALSE POSITIVE reproducer. Valid PHP: `Color::{$name}` is dynamic enum-case access on a defined enum, not a bare constant.
mir 0.42.0 currently emits (the bug): UndefinedConstant@8:11-8:16: ... `Color`
Expected: no issue. Remove ===ignore=== to activate once fixed.
===ignore===
===config===
php_version=8.4
===file===
<?php
enum Color: string {
    case Red = 'red';
    case Blue = 'blue';
}
function pick(string $name): Color {
    // FP expected: UndefinedConstant "Color" on dynamic enum-case access
    return Color::{$name};
}
===expect===
