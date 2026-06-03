===description===
Reused key var
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
PossiblyInvalidArrayOffset@8:5-8:34: Array offset might be invalid: expects 'array', got 'array<int, string>|false'
