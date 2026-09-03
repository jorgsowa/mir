===description===
FP-I1: the `xdebug` PECL extension (xdebug_break, xdebug_get_stack_depth,
...) had no vendored stubs/ dir despite PhpStormStubsMap.php already
listing every entry — same missing-stub root cause as the fixed C6 (ast).
===config===
suppress=UnusedVariable
===file===
<?php

function trace(): void {
    xdebug_break();
    $depth = xdebug_get_stack_depth();
}
===expect===
