===description===
$this->__construct() inside __wakeup must not trigger DirectConstructorCall.
This pattern is used in real PHP serialization code to re-establish resources
(e.g. database connections, file handles) after unserialization.
===file===
<?php
class DbConnection {
    private ?\PDO $pdo = null;

    public function __construct(
        private string $dsn,
        private string $user,
        private string $pass,
    ) {
        $this->pdo = new \PDO($this->dsn, $this->user, $this->pass);
    }

    public function __wakeup(): void {
        $this->__construct($this->dsn, $this->user, $this->pass);
    }
}
===expect===
