===description===
PossiblyInvalidArrayAccess does NOT fire when the type is a definite array.
===file===
<?php
$a = ["hello", "world"];
echo $a[0];
===expect===
