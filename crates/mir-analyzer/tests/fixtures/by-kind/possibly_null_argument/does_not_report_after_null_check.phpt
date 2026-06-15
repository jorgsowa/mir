===description===
does not report after null check
===file===
<?php
function greet(string $name): void {}

function test(?string $value): void {
    if ($value !== null) {
        greet($value);
    }
}
===expect===
UnusedParam@2:15-2:27: Parameter $name is never used
