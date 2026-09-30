===description===
Invalid array access
===file===
<?php
$a = 5;
echo $a[0];
//   ^^^^^ InvalidArrayAccess: Cannot use [] operator on non-array type '5'
===expect===
