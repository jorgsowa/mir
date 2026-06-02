===description===
Specific error message
===file===
<?php
$params = ["key" => "value"];
echo $params["fieldName"];
===expect===
NonExistentArrayOffset@3:14-3:25: Array offset 'fieldName' does not exist
