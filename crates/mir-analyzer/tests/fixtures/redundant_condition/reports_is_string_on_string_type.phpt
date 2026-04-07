===source===
<?php
function f(string $x): void {
    if (is_string($x)) {}
}
===expect===
RedundantCondition at 3:8
