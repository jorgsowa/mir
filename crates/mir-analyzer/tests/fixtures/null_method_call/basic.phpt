===source===
<?php
function test(): void {
    $x = null;
    $x->foo();
}
===expect===
NullMethodCall: Cannot call method foo() on null
