===description===
MixedAssignment fires when the right-hand side of an assignment resolves to mixed,
such as a mixed array element access.
===file===
<?php
/** @var array<string, mixed> $data */
$data = [];
$value = $data["key"];

===expect===
MixedAssignment@4:1-4:22: Variable $value is assigned a mixed type
