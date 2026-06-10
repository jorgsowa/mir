===description===
Mixed array offset
===file===
<?php
/** @var mixed */
$a = 5;
echo [1, 2, 3, 4][$a];
===expect===
MixedArrayOffset@4:19-4:21: Mixed type used as array offset
