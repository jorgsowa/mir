===description===
from nullable variable
===file===
<?php
function test(bool $flag): void {
    $x = $flag ? [1, 2, 3] : null;
    echo $x[0];
}
===expect===
PossiblyNullArrayAccess@4:9: Cannot access array on possibly null value
===ignore===
TODO
