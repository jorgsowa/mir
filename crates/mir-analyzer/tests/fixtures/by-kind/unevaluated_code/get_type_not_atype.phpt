===description===
Get type not a type
===file===
<?php
$a = rand(0, 10) ? 1 : "two";

switch (gettype($a)) {
    case "int":
//       ^^^^^ UnevaluatedCode: Unevaluated code: gettype() never returns "int" (did you mean "integer"?)
        break;
}
===expect===
