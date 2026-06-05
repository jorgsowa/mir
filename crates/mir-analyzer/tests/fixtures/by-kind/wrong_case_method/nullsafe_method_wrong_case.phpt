===description===
Wrong case method name via null-safe call is reported.
===file===
<?php
class Connection {
    public function getHandle(): mixed { return null; }
}
function getConn(): ?Connection { return null; }
$x = getConn()?->GETHANDLE();
===expect===
WrongCaseMethod@6:18-6:27: Method name 'Connection::GETHANDLE' has incorrect casing; use 'getHandle'
