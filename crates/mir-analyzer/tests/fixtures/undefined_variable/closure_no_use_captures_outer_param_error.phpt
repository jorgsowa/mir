===file===
<?php
function outer(string $x): callable {
    return function(): string {
        return $x;
    };
}
===expect===
UndefinedVariable: Variable $x is not defined
