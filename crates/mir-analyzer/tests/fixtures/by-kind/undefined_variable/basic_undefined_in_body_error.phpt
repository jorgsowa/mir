===description===
Using a variable that was never assigned in the same scope reports UndefinedVariable.
===config===
suppress=MixedReturnStatement
===file===
<?php
function foo(): string {
    return $result;
//         ^^^^^^^ UndefinedVariable: Variable $result is not defined
}
===expect===
