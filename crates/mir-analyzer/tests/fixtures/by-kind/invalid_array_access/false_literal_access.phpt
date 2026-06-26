===description===
InvalidArrayAccess fires when accessing a boolean false literal with []
===file===
<?php
$a = false;
echo $a[0];
===expect===
InvalidArrayAccess@3:5-3:10: Cannot use [] operator on non-array type 'false'
