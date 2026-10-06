===description===
InvalidArrayAccess fires inside the is_int branch of a variable declared as int|string
===file===
<?php
/** @var int|string $x */
$x = 5;
if (is_int($x)) {
    echo $x[0];
//       ^^^^^ InvalidArrayAccess: Cannot use [] operator on non-array type 'int'
}
