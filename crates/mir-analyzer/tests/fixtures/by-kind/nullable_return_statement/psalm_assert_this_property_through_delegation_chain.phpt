===description===
Delegation is followed across several hops, regardless of declaration order.
===file===
<?php
final class Conn {
    public int $lastId = 1;
}
final class Db {
    private ?Conn $connection = null;
    public function insert(): int {
        $this->outer();
        return $this->connection->lastId;
    }
    private function outer(): void {
        $this->inner();
    }
    private function inner(): void {
        $this->connect();
    }
    /** @psalm-assert Conn $this->connection */
    private function connect(): void {
        $this->connection = new Conn();
    }
}
