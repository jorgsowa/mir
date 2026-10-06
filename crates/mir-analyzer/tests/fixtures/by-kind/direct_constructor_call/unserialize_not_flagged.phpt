===description===
$this->__construct() inside the legacy Serializable::unserialize() method is a valid
re-initialization pattern and must not emit DirectConstructorCall.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Connection {
    private \PDO $pdo;

    public function __construct(private string $dsn) {
        $this->pdo = new \PDO($dsn);
    }

    public function unserialize(string $data): void {
        $this->__construct($data);
    }
}
