===source===
<?php
function test(): void {
    $x = null;
    $x->foo();
}
===expect===
NullMethodCall: $x->foo()
