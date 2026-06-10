===description===
Use duplicate name
===file===
<?php
$foo = "bar";

$a = function (string $foo) use ($foo) : string {
  return $foo;
};
===expect===
