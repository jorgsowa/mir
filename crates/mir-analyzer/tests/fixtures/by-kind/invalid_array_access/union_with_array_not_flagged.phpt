===description===
InvalidArrayAccess does NOT fire when the type includes an array variant.
===file===
<?php
/** @var array<int>|null $x */
$x = null;
if ($x !== null) {
    $val = $x[0];
}

===expect===
