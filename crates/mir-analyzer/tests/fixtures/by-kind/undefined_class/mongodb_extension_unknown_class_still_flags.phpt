===description===
Negative control for the mongodb stub vendoring fix (Sector I1): vendoring
the real stub files must not turn `MongoDB\*` into a wildcard-resolved
namespace — a class that isn't actually part of the extension must still
be flagged undefined.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function f(MongoDB\Driver\NotARealClass $x): void {}
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedClass: Class MongoDB\Driver\NotARealClass does not exist
===expect===
