===description===
Assert if true no annotation
===file===
<?php
function isValidString(?string $myVar) : bool {
    return $myVar !== null && $myVar[0] === "a";
}

$myString = rand(0, 1) ? "abacus" : null;

if (isValidString($myString)) {
    echo "Ma chaine " . $myString;
}
===expect===
PossiblyNullOperand
===ignore===
TODO
