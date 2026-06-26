===description===
InvalidArrayAccess fires when every type in a union is non-subscriptable (int|bool)
===file===
<?php
/** @var int|bool $x */
$x = 1;
echo $x[0];
===expect===
InvalidArrayAccess@4:5-4:10: Cannot use [] operator on non-array type 'int|bool'
