===source===
<?php
function greet(string $name): void {}

function test(?string $value): void {
    if ($value !== null) {
        greet($value);
    }
}
===expect===
UnusedParam: $name
