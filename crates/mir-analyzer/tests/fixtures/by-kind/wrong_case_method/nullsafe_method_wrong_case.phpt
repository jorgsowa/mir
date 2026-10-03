===description===
Wrong case method name via null-safe call is reported.
===config===
<mir>
  <issueHandlers>
    <MixedAssignment errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Connection {
    public function getHandle(): mixed { return null; }
}
function getConn(): ?Connection { return null; }
$x = getConn()?->GETHANDLE();
//               ^^^^^^^^^ WrongCaseMethod: Method name 'Connection::GETHANDLE' has incorrect casing; use 'getHandle'
===expect===
