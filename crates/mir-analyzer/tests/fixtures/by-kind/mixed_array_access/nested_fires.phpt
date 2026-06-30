===description===
MixedArrayAccess fires only on the innermost access when the root is mixed; the outer access does not re-emit the diagnostic.
===file===
<?php
/** @var mixed */
$a = [];
echo $a[0][1];
===expect===
MixedArrayAccess@4:5-4:10: Array access on mixed type
