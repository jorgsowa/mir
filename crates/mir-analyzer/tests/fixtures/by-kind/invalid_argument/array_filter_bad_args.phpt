===description===
Array filter bad args
===config===
suppress=UnusedParam
===file===
<?php
function foo(int $i) : bool {
  return true;
}

array_filter(["hello"], "foo");
===expect===
