===description===
A method that calls an asserting sibling as a top-level `$this->connect();`
statement carries that `$this->property` assertion to its own callers.
===file===
<?php
final class Conn {
    public int $lastId = 1;
}
final class Db {
    private ?Conn $connection = null;
    /** @psalm-assert Conn $this->connection */
    private function connect(): void {
        $this->connection = new Conn();
    }
    public function bulk(): void {
        $this->connect();
    }
    public function insert(): Conn {
        $this->bulk();
        $c = $this->connection;
        /** @mir-check $c is Conn */
        return $c;
    }
}
