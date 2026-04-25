===file===
<?php
function f(): int {
    $x = true ? 1 : 'hello';
    return $x;
}
===expect===
InvalidReturnType: Return type '1|"hello"' is not compatible with declared 'int'
