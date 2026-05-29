===description===
does not report call on null as undefined method
===file===
<?php
function test(): void {
    $x = null;
    $x->foo();
}
===expect===
NullMethodCall@4:5-4:14: Cannot call method foo() on null
