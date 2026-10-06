===description===
reports possibly null argument
===file===
<?php
function greet(string $name): void {}
//             ^^^^^^^^^^^^ UnusedParam: Parameter $name is never used

function test(?string $value): void {
    greet($value);
//        ^^^^^^ PossiblyNullArgument: Argument $name of greet() might be null
}
