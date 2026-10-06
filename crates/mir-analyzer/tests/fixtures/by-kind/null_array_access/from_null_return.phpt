===description===
NullArrayAccess fires when accessing an element of a value typed as null.
===file===
<?php
function nullReturn(): null {
    return null;
}
$x = nullReturn();
echo $x[0];
//   ^^^^^ NullArrayAccess: Cannot access array on null
