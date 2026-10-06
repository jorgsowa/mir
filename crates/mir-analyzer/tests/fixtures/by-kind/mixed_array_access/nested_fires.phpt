===description===
MixedArrayAccess fires only on the innermost access when the root is mixed; the outer access does not re-emit the diagnostic.
===file===
<?php
/** @var mixed */
$a = [];
echo $a[0][1];
//   ^^^^^ MixedArrayAccess: Array access on mixed type
