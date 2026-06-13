===description===
Possibly invalid array access with nested array
===config===
suppress=UnusedVariable
===file===
<?php
/**
 * @return array<int,array<string,float>>|string
 */
function return_array() {
    return rand() % 5 > 3 ? [["key" => 3.5]] : "key:3.5";
}
$result = return_array();
$v = $result[0]["key"];
===expect===
