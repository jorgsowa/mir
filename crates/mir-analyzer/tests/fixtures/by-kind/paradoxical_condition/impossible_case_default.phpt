===description===
Impossible case default
===file===
<?php
$a = rand(0, 1) ? "a" : "b";

switch ($a) {
    case "a":
        break;

    case "b":
        break;

    default:
        echo "impossible";
}
===expect===
