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
        echo "impossible";
}
===expect===
TypeDoesNotContainType@11:9-11:12: Type '"a"|"b"' can never contain type '"c"'
