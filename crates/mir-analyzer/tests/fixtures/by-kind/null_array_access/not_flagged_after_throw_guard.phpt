===description===
PossiblyNullArrayAccess does NOT fire when a throw-based guard narrows the type
to non-null before the array access.
===file===
<?php
function test(?array $arr): void {
    if ($arr === null) {
        throw new \InvalidArgumentException('arr required');
    }
    echo $arr[0];
}
===expect===
