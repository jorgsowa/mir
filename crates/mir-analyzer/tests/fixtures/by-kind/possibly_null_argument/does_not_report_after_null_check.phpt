===description===
does not report after null check
===file===
<?php
function greet(string $name): void {}
//             ^^^^^^^^^^^^ UnusedParam: Parameter $name is never used

function test(?string $value): void {
    if ($value !== null) {
        greet($value);
    }
}
===expect===
