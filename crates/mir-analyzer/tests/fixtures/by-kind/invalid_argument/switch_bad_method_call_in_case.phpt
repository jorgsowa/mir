===description===
Switch bad method call in case
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function f(string $p): void { }

switch (true) {
    case $q = (bool) rand(0,1):
        f($q); // this type problem is not detected
//        ^^ InvalidArgument: Argument $p of f() expects 'string', got 'bool'
        break;
}
