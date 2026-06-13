===description===
Use duplicate name
===config===
suppress=UnusedVariable
===file===
<?php
$foo = "bar";

$a = function (string $foo) use ($foo) : string {
  return $foo;
};
===expect===
