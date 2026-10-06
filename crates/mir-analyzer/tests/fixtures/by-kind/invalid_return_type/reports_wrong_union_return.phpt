===description===
reports wrong union return
===file===
<?php
function f(): int {
    $x = true ? 1 : 'hello';
    return $x;
//  ^^^^^^^^^^ InvalidReturnType: Return type '1|"hello"' is not compatible with declared 'int'
}
