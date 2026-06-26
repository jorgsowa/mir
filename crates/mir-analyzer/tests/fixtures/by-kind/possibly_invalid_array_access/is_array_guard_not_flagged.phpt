===description===
PossiblyInvalidArrayAccess does NOT fire after an is_array() guard narrows the
type to a pure array — the invalid int atom is excluded in the true branch.
===file===
<?php
$a = rand(0, 1) > 0 ? 5 : ["hello"];
if (is_array($a)) {
    echo $a[0];
}
===expect===
