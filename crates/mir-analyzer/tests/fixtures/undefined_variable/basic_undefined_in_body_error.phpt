===description===
Using a variable that was never assigned in the same scope reports UndefinedVariable.
===file===
<?php
function foo(): string {
    return $result;
}
===expect===
UndefinedVariable@3:12: Variable $result is not defined
