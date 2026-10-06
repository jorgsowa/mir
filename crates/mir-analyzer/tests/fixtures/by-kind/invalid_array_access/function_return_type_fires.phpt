===description===
InvalidArrayAccess fires when array access is applied to a variable holding a function's int return value
===file===
<?php
function getNumber(): int
{
    return 42;
}
$n = getNumber();
echo $n[0];
//   ^^^^^ InvalidArrayAccess: Cannot use [] operator on non-array type 'int'
