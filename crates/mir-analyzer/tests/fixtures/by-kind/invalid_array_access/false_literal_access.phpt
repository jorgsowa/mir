===description===
InvalidArrayAccess fires when accessing a boolean false literal with []
===file===
<?php
$a = false;
echo $a[0];
//   ^^^^^ InvalidArrayAccess: Cannot use [] operator on non-array type 'false'
