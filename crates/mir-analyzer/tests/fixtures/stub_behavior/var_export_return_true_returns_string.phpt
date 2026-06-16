===description===
var_export with $return=true returns string, not string|null
===config===
suppress=UnusedVariable
===file===
<?php
$exported = var_export(['key' => 'value'], true);
/** @mir-check $exported is string */
echo $exported;
===expect===
