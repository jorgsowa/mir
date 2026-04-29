===description===
A ternary that may produce null yields a possibly-null type, reporting
PossiblyNullMethodCall instead of NullMethodCall. An UndefinedMethod
diagnostic is also raised because stdClass::foo() does not exist.
===file===
<?php
function test(bool $flag): void {
    $x = $flag ? new stdClass() : null;
    $x->foo();
}
===expect===
PossiblyNullMethodCall: Cannot call method foo() on possibly null value
UndefinedMethod: Method stdClass::foo() does not exist
