===description===
A function annotated with @deprecated reports DeprecatedCall at its call site,
including the deprecation message from the docblock.
===file===
<?php
/** @deprecated use newGreet() instead */
function oldGreet(string $name): void {}

function test(): void {
    oldGreet('Alice');
}
===expect===
UnusedParam@3:18-3:30: Parameter $name is never used
DeprecatedCall@6:4-6:21: Call to deprecated function oldGreet: use newGreet() instead
