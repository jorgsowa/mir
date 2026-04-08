===source===
<?php
function foo(): int {
    $unused = 1;
    return 42;
}
===expect===
UnusedVariable: <no snippet>
