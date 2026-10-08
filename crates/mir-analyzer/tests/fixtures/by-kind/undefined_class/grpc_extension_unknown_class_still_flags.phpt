===description===
Vendoring the grpc stubs must not turn `Grpc\*` into a wildcard namespace: a class
that is not part of the extension is still undefined.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function f(Grpc\NotARealClass $x): void {}
//         ^^^^^^^^^^^^^^^^^^ UndefinedClass: Class Grpc\NotARealClass does not exist
