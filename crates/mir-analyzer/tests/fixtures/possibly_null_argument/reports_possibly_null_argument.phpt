===source===
<?php
function greet(string $name): void {}

function test(?string $value): void {
    greet($value);
}
===expect===
UnusedParam: Parameter $name is never used
PossiblyNullArgument: Argument $name of greet() might be null
