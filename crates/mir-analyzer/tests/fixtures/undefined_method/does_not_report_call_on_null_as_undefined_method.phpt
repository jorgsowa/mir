===description===
does not report call on null as undefined method
===file===
<?php
function test(): void {
    $x = null;
    $x->foo();
}
===expect===
NullMethodCall: Cannot call method foo() on null
===ignore===
TODO
