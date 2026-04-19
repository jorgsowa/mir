===source===
<?php
interface Iterator {}
interface Countable {}

function f(): Iterator&Countable {
    /** @var Iterator&Countable $x */
    $x = null;
    return $x;
}
===expect===
