===source===
<?php
function f(string|int $x): void {
    if (is_string($x)) {}
}
===expect===
