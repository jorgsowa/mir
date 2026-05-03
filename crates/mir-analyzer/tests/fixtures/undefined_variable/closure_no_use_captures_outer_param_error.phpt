===description===
closure no use captures outer param error
===file===
<?php
function outer(string $x): callable {
    return function(): string {
        return $x;
    };
}
===expect===
UndefinedVariable@4:15: Variable $x is not defined
===ignore===
TODO
