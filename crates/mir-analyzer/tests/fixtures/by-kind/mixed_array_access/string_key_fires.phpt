===description===
MixedArrayAccess fires when using a string key on a mixed-typed variable.
===file===
<?php
/** @var mixed */
$a = [];
echo $a['key'];
===expect===
MixedArrayAccess@4:5-4:14: Array access on mixed type
