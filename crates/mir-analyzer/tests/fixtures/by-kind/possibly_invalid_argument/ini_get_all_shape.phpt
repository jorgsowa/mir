===description===
`ini_get_all(null, false)` retains string keys when its value type is mixed.
===file===
<?php
foreach (ini_get_all(null, false) as $key => $_value) {
    echo strtolower($key);
}
===expect===
MixedAssignment@2:45-2:52: Variable $_value is assigned a mixed type
