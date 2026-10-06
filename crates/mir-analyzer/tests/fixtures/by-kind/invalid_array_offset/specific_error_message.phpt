===description===
Specific error message
===file===
<?php
$params = ["key" => "value"];
echo $params["fieldName"];
//           ^^^^^^^^^^^ NonExistentArrayOffset: Array offset 'fieldName' does not exist
