===source===
<?php
/** @deprecated use newGreet() instead */
function oldGreet(string $name): void {}

function test(): void {
    oldGreet('Alice');
}
===expect===
UnusedParam: $name
DeprecatedCall: oldGreet('Alice')
