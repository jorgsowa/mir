===description===
closure no use captures outer param error
===config===
suppress=MixedReturnStatement
===file===
<?php
function outer(string $x): callable {
    return function(): string {
        return $x;
    };
}
===expect===
UndefinedVariable@4:15-4:17: Variable $x is not defined
