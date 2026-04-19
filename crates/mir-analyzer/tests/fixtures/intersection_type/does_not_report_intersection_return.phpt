===source===
<?php
interface Countable {}
interface Iterator {}

/** @param Iterator&Countable $x */
function f($x): void {
    $_ = $x;
}
===expect===
