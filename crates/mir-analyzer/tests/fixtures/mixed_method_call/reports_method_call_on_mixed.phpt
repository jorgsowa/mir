===description===
reports method call on mixed
===file===
<?php
function test(mixed $value): void {
    $value->someMethod();
}
===expect===
MixedMethodCall@3:4: Method someMethod() called on mixed type
===ignore===
TODO
