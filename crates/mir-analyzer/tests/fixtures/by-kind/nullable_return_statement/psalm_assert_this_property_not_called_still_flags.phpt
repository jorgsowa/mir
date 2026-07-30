===description===
Negative control for the `$this->property` assertion fix: without ever
calling the asserting method, the nullable property access must still be
flagged — the fix only applies the assertion after an actual call.
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
    public function insertWithoutConnect(): int {
        return $this->connection->lastId;
    }
}
===expect===
NullableReturnStatement@12:8-12:41: Return type 'int|null' is not compatible with declared 'int'
PossiblyNullPropertyFetch@12:15-12:40: Cannot access property $lastId on possibly null value
