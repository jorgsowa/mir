===description===
NullMethodCall does NOT fire when the object is a definite non-null type.
===file===
<?php
function test(): void {
    $x = new stdClass();
    $x->foo = "bar";
}
===expect===
