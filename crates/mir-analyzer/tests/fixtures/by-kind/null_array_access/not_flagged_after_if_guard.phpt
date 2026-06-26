===description===
PossiblyNullArrayAccess does NOT fire when a null check guards the array access.
===file===
<?php
function test(?array $arr): void {
    if ($arr !== null) {
        echo $arr[0];
    }
}
===expect===
