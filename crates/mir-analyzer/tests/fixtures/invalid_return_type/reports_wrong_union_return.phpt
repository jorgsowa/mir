===source===
<?php
function f(): int {
    $x = true ? 1 : 'hello';
    return $x;
}
===expect===
InvalidReturnType: return $x;
