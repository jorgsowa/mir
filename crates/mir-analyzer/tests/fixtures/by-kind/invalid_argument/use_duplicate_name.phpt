===description===
Use duplicate name
===ignore===
TODO
===file===
<?php
$foo = "bar";

$a = function (string $foo) use ($foo) : string {
  return $foo;
};
===expect===
