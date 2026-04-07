===source===
<?php
function f(string $x): void {
    if ($x === null) {}
}
===expect===
RedundantCondition at 3:8
