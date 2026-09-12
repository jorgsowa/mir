===description===
`ini_get_all()` retains string keys when its value type is mixed.
===file===
<?php
foreach (ini_get_all() as $key => $_value) {
    echo strtolower($key);
}
===expect===
MixedAssignment@2:34-2:41: Variable $_value is assigned a mixed type
