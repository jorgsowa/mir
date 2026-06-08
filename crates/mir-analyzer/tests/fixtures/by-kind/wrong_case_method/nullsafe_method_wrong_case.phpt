===description===
Wrong case method name via null-safe call is reported.
===file===
<?php
class Connection {
    public function getHandle(): mixed { return null; }
}
function getConn(): ?Connection { return null; }
getConn()?->GETHANDLE();
===expect===
WrongCaseMethod@6:13-6:22: Method name 'Connection::GETHANDLE' has incorrect casing; use 'getHandle'
