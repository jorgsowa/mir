===description===
reports possibly null argument
===file===
<?php
function greet(string $name): void {}

function test(?string $value): void {
    greet($value);
}
===expect===
UnusedParam@2:15: Parameter $name is never used
PossiblyNullArgument@5:10: Argument $name of greet() might be null
===ignore===
TODO
