===description===
Foreach over mixed emits MixedAssignment for value variable
===file===
<?php
/** @var mixed */
$arr = [1, 2, 3];
foreach ($arr as $v) {
//               ^^ MixedAssignment: Variable $v is assigned a mixed type
    echo $v;
}
