===description===
Same `$this->property` assertion target, but the asserting method also
takes its own regular parameters — the receiver-targeted assertion must
still apply alongside ordinary param-targeted ones.
===file===
<?php
final class Conn {
    public int $lastId = 1;
}
final class Db {
    public ?Conn $connection = null;
    /** @psalm-assert Conn $this->connection */
    public function connect(string $dsn): void {
        $this->connection = new Conn();
    }
    public function insert(): int {
        $this->connect('mysql:host=localhost');
        return $this->connection->lastId;
    }
}
===expect===
UnusedParam@8:28-8:39: Parameter $dsn is never used
