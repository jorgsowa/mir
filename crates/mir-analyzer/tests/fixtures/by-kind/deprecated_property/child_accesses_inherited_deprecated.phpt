===description===
DeprecatedProperty fires when a child class instance accesses a deprecated property declared on the parent.
===file===
<?php
class Connection {
    /**
     * @deprecated Use $timeout_ms instead.
     */
    public int $timeout = 30;
}

class DbConnection extends Connection {}

$db = new DbConnection();
echo $db->timeout;
===expect===
DeprecatedProperty@12:10-12:17: Property DbConnection::$timeout is deprecated: Use $timeout_ms instead.
