===description===
Wrong case method name via null-safe call is reported.
===config===
suppress=MixedAssignment,UnusedVariable
===file===
<?php
class Connection {
    public function getHandle(): mixed { return null; }
}
function getConn(): ?Connection { return null; }
$x = getConn()?->GETHANDLE();
===expect===
WrongCaseMethod@6:17-6:26: Method name 'Connection::GETHANDLE' has incorrect casing; use 'getHandle'
