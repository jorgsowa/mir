===description===
reusedKeyVar
===file===
<?php
$key = "a";
echo $key;

$arr = ["foo" => "foo.foo"];

foreach ($arr as $key => $v) {
    list($key) = explode(".", $v);
    echo $key;
}
===expect===
UnusedVariable
===ignore===
TODO
