===description===
InvalidArrayAccess fires when accessing a boolean true literal with []
===file===
<?php
$a = true;
echo $a[0];
===expect===
InvalidArrayAccess@3:5-3:10: Cannot use [] operator on non-array type 'true'
