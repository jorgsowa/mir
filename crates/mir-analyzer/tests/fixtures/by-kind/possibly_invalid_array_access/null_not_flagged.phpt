===description===
PossiblyInvalidArrayAccess does NOT fire for a null|array union — null is not
in the invalid-for-access set; PossiblyNullArrayAccess is the applicable
diagnostic for nullable arrays.
===file===
<?php
$a = rand(0, 1) > 0 ? null : ["hello"];
echo $a[0];
===expect===
PossiblyNullArrayAccess@3:5-3:10: Cannot access array on possibly null value
