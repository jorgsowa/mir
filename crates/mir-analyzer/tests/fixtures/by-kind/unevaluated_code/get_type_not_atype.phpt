===description===
Get type not a type
===file===
<?php
$a = rand(0, 10) ? 1 : "two";

switch (gettype($a)) {
    case "int":
        break;
}
===expect===
UnevaluatedCode@5:9-5:14: Unevaluated code: gettype() never returns "int" (did you mean "integer"?)
