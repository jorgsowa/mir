===description===
backed enum int value returned as string
===file===
<?php
enum Color: int {
    case Red = 1;
    case Blue = 2;
}
function test(Color $color): string {
    return $color->value;
}
===expect===
InvalidReturnType@7:4: Return type 'int' is not compatible with declared 'string'
===ignore===
TODO
