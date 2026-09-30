===description===
Impossible case value
===file===
<?php
$a = rand(0, 1) ? "a" : "b";

switch ($a) {
    case "a":
        break;

    case "b":
        break;

    case "c":
//       ^^^ TypeDoesNotContainType: Type '"a"|"b"' can never contain type '"c"'
        echo "impossible";
}
===expect===
