===source===
<?php
function f(string $x): void {
    if ($x !== null) {}
}
===expect===
RedundantCondition: $x !== null
