===description===
Foreach over mixed emits MixedAssignment for value variable
===file===
<?php
/** @var mixed */
$arr = [1, 2, 3];
foreach ($arr as $v) {
    echo $v;
}
===expect===
MixedAssignment@4:18-4:20: Variable $v is assigned a mixed type
