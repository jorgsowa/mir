===description===
Bare callable (no param spec) invoked with args must not fire TooManyArguments
===file===
<?php
function process(callable $callback): void {
    $callback('arg');
}

function processMulti(callable $fn): void {
    $fn('a', 'b', 'c');
}

===expect===
