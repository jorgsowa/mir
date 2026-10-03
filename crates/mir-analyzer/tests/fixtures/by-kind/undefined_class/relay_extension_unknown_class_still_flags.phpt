===description===
Negative control for the relay stub vendoring fix (Sector I1): vendoring
the real stub files must not turn `Relay\*` into a wildcard-resolved
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
function f(Relay\NotARealClass $x): void {}
//         ^^^^^^^^^^^^^^^^^^^ UndefinedClass: Class Relay\NotARealClass does not exist
===expect===
