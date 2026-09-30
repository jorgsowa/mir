===description===
InvalidArrayAccess fires when accessing a boolean true literal with []
===file===
<?php
$a = true;
echo $a[0];
//   ^^^^^ InvalidArrayAccess: Cannot use [] operator on non-array type 'true'
===expect===
