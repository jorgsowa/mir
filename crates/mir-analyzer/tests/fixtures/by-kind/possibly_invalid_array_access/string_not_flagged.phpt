===description===
PossiblyInvalidArrayAccess does NOT fire for a string|array union — PHP allows
subscript access on strings (character at index), so string is not an invalid
array-access type.
===file===
<?php
$a = rand(0, 1) > 0 ? "hello" : ["world"];
echo $a[0];
===expect===
