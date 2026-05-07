===description===
Calling a method on a variable assigned null reports NullMethodCall.
===file===
<?php
function test(): void {
    $x = null;
    $x->foo();
}
===expect===
NullMethodCall@4:4: Cannot call method foo() on null
