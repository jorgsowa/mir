===description===
NullArrayAccess fires when accessing an element of a value typed as null.
===file===
<?php
function nullReturn(): null {
    return null;
}
$x = nullReturn();
echo $x[0];
===expect===
NullArrayAccess@6:5-6:10: Cannot access array on null
