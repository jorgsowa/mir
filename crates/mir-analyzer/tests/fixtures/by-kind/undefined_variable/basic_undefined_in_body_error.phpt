===description===
Using a variable that was never assigned in the same scope reports UndefinedVariable.
===config===
suppress=MixedReturnStatement
===file===
<?php
function foo(): string {
    return $result;
}
===expect===
UndefinedVariable@3:11-3:18: Variable $result is not defined
