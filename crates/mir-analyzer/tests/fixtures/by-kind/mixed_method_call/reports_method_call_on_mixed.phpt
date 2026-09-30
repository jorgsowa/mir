===description===
reports method call on mixed
===file===
<?php
function test(mixed $value): void {
    $value->someMethod();
//  ^^^^^^^^^^^^^^^^^^^^ MixedMethodCall: Method someMethod() called on mixed type
}
===expect===
