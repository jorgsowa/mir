===description===
Assert if false no annotation
===file===
<?php
function isInvalidString(?string $myVar) : bool {
    return $myVar === null || $myVar[0] !== "a";
}

$myString = rand(0, 1) ? "abacus" : null;

if (isInvalidString($myString)) {
    // do something
} else {
    echo "Ma chaine " . $myString;
}
===expect===
