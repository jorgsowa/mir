===description===
NullArrayAccess does NOT fire after a null coalescing assignment resolves
a possibly-null value to a concrete array.
===file===
<?php
function test(?array $arr): void {
    $arr = $arr ?? [];
    echo $arr[0];
}
===expect===
