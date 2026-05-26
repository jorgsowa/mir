===description===
Possibly null array access
===file===
<?php
$a = rand(0, 1) ? [1, 2] : null;
echo $a[0];
===expect===
PossiblyNullArrayAccess
===ignore===
TODO
