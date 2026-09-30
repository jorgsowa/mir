===description===
Accessing a key that does not exist in a closed keyed array
===file===
<?php
$params = ["key" => "value"];
echo $params["fieldName"];
//           ^^^^^^^^^^^ NonExistentArrayOffset: Array offset 'fieldName' does not exist
===expect===
