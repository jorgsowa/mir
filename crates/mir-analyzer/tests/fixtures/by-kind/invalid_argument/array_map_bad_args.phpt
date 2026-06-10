===description===
Array map bad args
===ignore===
TODO
===file===
<?php
function foo(int $i) : bool {
  return true;
}

array_map("foo", ["hello"]);
===expect===
