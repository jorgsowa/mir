===description===
$this->__construct() inside __wakeup is a valid re-initialization pattern and must not emit DirectConstructorCall.
===file===
<?php
class Connection {
    private \PDO $pdo;

    public function __construct(private string $dsn) {
        $this->pdo = new \PDO($dsn);
    }

    public function __wakeup(): void {
        $this->__construct($this->dsn);
    }
}
===expect===
