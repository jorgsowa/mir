===description===
Accessing a key that does not exist in a closed keyed array
===file===
<?php
$params = ["key" => "value"];
echo $params["fieldName"];
===expect===
NonExistentArrayOffset@3:14-3:25: Array offset 'fieldName' does not exist
