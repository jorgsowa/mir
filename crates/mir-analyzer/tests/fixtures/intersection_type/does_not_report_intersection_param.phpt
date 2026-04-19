===source===
<?php
interface Countable {}
interface Iterator {}

function f(Iterator&Countable $x): void {
    $_ = $x;
}
===expect===
