===source===
<?php
function test(): void {
    /** @var mixed $x */
    $x = 1;
    $x->anything();
}
===expect===
MixedMethodCall: Method anything() called on mixed type
