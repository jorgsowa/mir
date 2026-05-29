===description===
reports possibly null argument
===file===
<?php
function greet(string $name): void {}

function test(?string $value): void {
    greet($value);
}
===expect===
UnusedParam@2:16-2:28: Parameter $name is never used
PossiblyNullArgument@5:11-5:17: Argument $name of greet() might be null
