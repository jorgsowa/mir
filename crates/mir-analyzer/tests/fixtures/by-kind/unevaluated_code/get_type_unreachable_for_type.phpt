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
//           ^^^^^^^^ UnevaluatedCode: Unevaluated code: gettype() of int never returns "string"
            break;
    }
}
===expect===
