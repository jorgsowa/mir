===description===
does not report method call on mixed
===file===
<?php
function test(): void {
    /** @var mixed $x */
    $x = 1;
    $x->anything();
//  ^^^^^^^^^^^^^^ MixedMethodCall: Method anything() called on mixed type
}
===expect===
