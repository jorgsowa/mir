===source===
<?php
function test(): void {
    /** @var mixed $x */
    $x = 1;
    $x->anything();
}
===expect===
MixedMethodCall at 5:4
