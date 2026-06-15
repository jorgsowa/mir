===description===
Invalid array access
===file===
<?php
$a = 5;
echo $a[0];
===expect===
InvalidArrayAccess@3:5-3:10: Cannot use [] operator on non-array type '5'
