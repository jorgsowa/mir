===description===
Negative control for the snmp stub vendoring fix (Sector I1): vendoring the
real stub file must not wildcard-resolve unrelated top-level class names —
a class that isn't actually part of the extension must still be flagged
undefined.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function f(NotARealSNMPClass $x): void {}
//         ^^^^^^^^^^^^^^^^^ UndefinedClass: Class NotARealSNMPClass does not exist
