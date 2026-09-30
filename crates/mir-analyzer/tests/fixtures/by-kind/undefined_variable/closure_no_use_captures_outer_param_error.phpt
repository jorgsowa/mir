===description===
closure no use captures outer param error
===config===
suppress=MixedReturnStatement
===file===
<?php
function outer(string $x): callable {
    return function(): string {
        return $x;
//             ^^ UndefinedVariable: Variable $x is not defined
    };
}
===expect===
