===description===
InvalidArrayAccess fires inside the is_int branch of a variable declared as int|string
===file===
<?php
/** @var int|string $x */
$x = 5;
if (is_int($x)) {
    echo $x[0];
}
===expect===
InvalidArrayAccess@5:9-5:14: Cannot use [] operator on non-array type 'int'
