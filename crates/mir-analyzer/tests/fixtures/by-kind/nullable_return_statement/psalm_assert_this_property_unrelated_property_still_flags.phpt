===description===
Negative control for the `$this->property` assertion fix: asserting one
property must not narrow an unrelated one — the fix targets exactly the
named property, not the whole receiver.
===file===
<?php
final class Conn {
    public int $lastId = 1;
}
final class Db {
    public ?Conn $connection = null;
    public ?Conn $backupConnection = null;
    /** @psalm-assert Conn $this->connection */
    public function connect(): void {
        $this->connection = new Conn();
    }
    public function insert(): int {
        $this->connect();
        return $this->backupConnection->lastId;
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'int|null' is not compatible with declared 'int'
//             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $lastId on possibly null value
    }
}
===expect===
