===source===
<?php
function f(string|int $x): void {
    if (is_string($x)) {
        if (is_string($x)) {}
    }
}
===expect===
RedundantCondition at 4:12
