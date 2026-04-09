===source===
<?php
function test(bool $flag): void {
    $x = $flag ? new stdClass() : null;
    $x->foo();
}
===expect===
PossiblyNullMethodCall: $x->foo()
UndefinedMethod: $x->foo()
