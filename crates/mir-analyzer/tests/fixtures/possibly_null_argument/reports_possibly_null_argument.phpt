===source===
<?php
function greet(string $name): void {}

function test(?string $value): void {
    greet($value);
}
===expect===
UnusedParam: $name
PossiblyNullArgument: $value
