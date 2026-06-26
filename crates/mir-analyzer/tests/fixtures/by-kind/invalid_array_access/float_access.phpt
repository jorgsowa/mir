===description===
InvalidArrayAccess fires when accessing a float literal with []
===file===
<?php
$a = 1.5;
echo $a[0];
===expect===
InvalidArrayAccess@3:5-3:10: Cannot use [] operator on non-array type '1.5'
