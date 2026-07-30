===description===
`@psalm-assert Type $this->property` on a method must narrow the CALL'S
OWN receiver's property, not a declared parameter — `$this` in the
docblock is written from the method's own perspective. Covers both a
zero-arg "connect-then-use" helper and a receiver that isn't `$this`
itself (a plain object-typed parameter).
===file===
<?php
final class Conn {
    public int $lastId = 1;
}
final class Db {
    public ?Conn $connection = null;
    /** @psalm-assert Conn $this->connection */
    public function connect(): void {
        $this->connection = new Conn();
    }
    public function insert(): int {
        $this->connect();
        return $this->connection->lastId;
    }
}
function useDb(Db $db): int {
    $db->connect();
    return $db->connection->lastId;
}
===expect===
