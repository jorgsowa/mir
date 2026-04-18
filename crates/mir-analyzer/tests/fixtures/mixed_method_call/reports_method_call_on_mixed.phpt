===source===
<?php
function test(mixed $value): void {
    $value->someMethod();
}
===expect===
MixedMethodCall: $value->someMethod()
