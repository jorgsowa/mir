===source===
<?php
/** @deprecated use newGreet() instead */
function oldGreet(string $name): void {}

function test(): void {
    oldGreet('Alice');
}
===expect===
UnusedParam: Parameter $name is never used
DeprecatedCall: Call to deprecated function oldGreet
