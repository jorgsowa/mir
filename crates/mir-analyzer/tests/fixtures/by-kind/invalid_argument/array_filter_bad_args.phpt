===description===
Array filter bad args
===ignore===
TODO
===file===
<?php
function foo(int $i) : bool {
  return true;
}

array_filter(["hello"], "foo");
===expect===
