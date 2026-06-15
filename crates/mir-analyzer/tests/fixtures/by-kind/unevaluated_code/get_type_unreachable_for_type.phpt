===description===
gettype switch arm unreachable for the argument's inferred type
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php
function scope(int $n): void {
    switch (gettype($n)) {
        case "integer":
            break;
        case "string":
            break;
    }
}
===expect===
UnevaluatedCode@6:13-6:21: Unevaluated code: gettype() of int never returns "string"
