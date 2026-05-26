===description===
Array map bad args
===file===
<?php
function foo(int $i) : bool {
  return true;
}

array_map("foo", ["hello"]);
===expect===
InvalidScalarArgument
===ignore===
TODO
