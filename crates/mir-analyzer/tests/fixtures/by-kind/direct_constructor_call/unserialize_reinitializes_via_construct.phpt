===description===
$this->__construct() inside the legacy Serializable::unserialize() method must not trigger
DirectConstructorCall. This is the standard, PHP-manual-documented re-initialization idiom for
classes implementing the (deprecated but still supported) Serializable interface.
===config===
suppress=UnusedParam
===file===
<?php
class DbConnection implements \Serializable {
    private ?\PDO $pdo = null;

    public function __construct(
        private string $dsn,
        private string $user,
        private string $pass,
    ) {
        $this->pdo = new \PDO($this->dsn, $this->user, $this->pass);
    }

    public function serialize(): string {
        return serialize([$this->dsn, $this->user, $this->pass]);
    }

    public function unserialize(string $data): void {
        $this->__construct($this->dsn, $this->user, $this->pass);
    }
}
===expect===
