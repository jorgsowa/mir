===file===
<?php
function test(mixed $value): void {
    $value->someMethod();
}
===expect===
MixedMethodCall: Method someMethod() called on mixed type
