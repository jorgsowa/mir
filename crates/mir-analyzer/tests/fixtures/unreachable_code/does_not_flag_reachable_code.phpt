===description===
does not flag reachable code
===file===
<?php
function foo(): int {
    $x = 2;
    return $x;
}
===expect===
===ignore===
TODO
