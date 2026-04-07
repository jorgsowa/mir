===source===
<?php
function test(): void {
    $x = null;
    $x->foo();
}
===expect===
NullMethodCall at 4:4
