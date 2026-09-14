===description===
`ini_get_all(null, false)` retains string keys with a precise value type.
===file===
<?php
foreach (ini_get_all(null, false) as $key => $_value) {
    echo strtolower($key);
}
===expect===
