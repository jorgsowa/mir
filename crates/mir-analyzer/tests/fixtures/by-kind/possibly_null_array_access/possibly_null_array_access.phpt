===description===
Possibly null array access
===file===
<?php
$a = rand(0, 1) ? [1, 2] : null;
echo $a[0];
===expect===
PossiblyNullArrayAccess@3:6-3:11: Cannot access array on possibly null value
