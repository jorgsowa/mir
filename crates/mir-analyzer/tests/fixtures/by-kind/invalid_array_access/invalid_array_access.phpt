===description===
Invalid array access
===file===
<?php
$a = 5;
echo $a[0];
===expect===
InvalidArrayAccess@3:6-3:11: Cannot use [] operator on non-array type '5'
