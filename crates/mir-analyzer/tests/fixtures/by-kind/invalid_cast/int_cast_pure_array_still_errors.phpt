===description===
(int) cast on a pure array type still emits InvalidCast — no scalar-safe atoms present
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function getArray(): array {
    return [];
}

$x = (int) getArray();
//         ^^^^^^^^^^ InvalidCast: Cannot cast 'array' to 'int'
