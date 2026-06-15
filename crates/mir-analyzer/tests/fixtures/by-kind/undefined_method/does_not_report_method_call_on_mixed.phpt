===description===
does not report method call on mixed
===file===
<?php
function test(): void {
    /** @var mixed $x */
    $x = 1;
    $x->anything();
}
===expect===
MixedMethodCall@5:4-5:18: Method anything() called on mixed type
