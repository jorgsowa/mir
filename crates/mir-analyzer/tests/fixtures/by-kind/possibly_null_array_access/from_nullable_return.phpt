===description===
Unguarded access on the return value of a function declared ?array fires PossiblyNullArrayAccess
===file===
<?php
function getItems(): ?array {
    return null;
}

$arr = getItems();
echo $arr[0];
//   ^^^^^^^ PossiblyNullArrayAccess: Cannot access array on possibly null value
