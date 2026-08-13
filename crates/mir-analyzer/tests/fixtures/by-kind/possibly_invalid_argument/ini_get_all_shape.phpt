===description===
ini_get_all() stub returns a known-shaped array so foreach key usage reports no PossiblyInvalidArgument
===file===
<?php
foreach (ini_get_all() as $key => $_value) {
    echo strtolower($key);
}
===expect===
