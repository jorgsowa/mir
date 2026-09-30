===description===
Negative control: the delegating method resets the property after the call,
so the assertion must not reach its callers.
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
    public function reset(): void {
        $this->connect();
        $this->connection = null;
    }
    public function insert(): int {
        $this->reset();
        return $this->connection->lastId;
    }
}
===expect===
NullableReturnStatement@17:8-17:41: Return type 'int|null' is not compatible with declared 'int'
PossiblyNullPropertyFetch@17:15-17:40: Cannot access property $lastId on possibly null value
