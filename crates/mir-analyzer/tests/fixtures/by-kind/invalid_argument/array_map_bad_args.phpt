===description===
Array map bad args
===config===
suppress=UnusedParam
===file===
<?php
function foo(int $i) : bool {
  return true;
}

array_map("foo", ["hello"]);
===expect===
