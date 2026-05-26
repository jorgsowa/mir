===description===
backed enum name
===file===
<?php
enum Color: int {
    case Red = 1;
    case Blue = 2;
}
function test(?Color $color): string {
    return $color?->name ?? 'none';
}
===expect===
===ignore===
TODO
