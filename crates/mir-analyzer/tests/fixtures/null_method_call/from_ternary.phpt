===file===
<?php
function test(bool $flag): void {
    $x = $flag ? new stdClass() : null;
    $x->foo();
}
===expect===
PossiblyNullMethodCall: Cannot call method foo() on possibly null value
UndefinedMethod: Method stdClass::foo() does not exist
