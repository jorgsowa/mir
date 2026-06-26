===description===
PossiblyNullArrayAccess fires when accessing an element of a nullable array
parameter without a null guard.
===file===
<?php
function test(?array $arr): void {
    echo $arr[0];
}
===expect===
PossiblyNullArrayAccess@3:9-3:16: Cannot access array on possibly null value
