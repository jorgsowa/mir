===description===
Negative control: a conditional call, or a callee without an assertion,
proves nothing about the property.
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
    private function noop(): void {
    }
    public function maybe(bool $b): void {
        if ($b) {
            $this->connect();
        }
    }
    public function plain(): void {
        $this->noop();
    }
    public function viaMaybe(bool $b): int {
        $this->maybe($b);
        return $this->connection->lastId;
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'int|null' is not compatible with declared 'int'
//             ^^^^^^^^^^^^^^^^^^^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $lastId on possibly null value
    }
    public function viaPlain(): int {
        $this->plain();
        return $this->connection->lastId;
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ NullableReturnStatement: Return type 'int|null' is not compatible with declared 'int'
//             ^^^^^^^^^^^^^^^^^^^^^^^^^ PossiblyNullPropertyFetch: Cannot access property $lastId on possibly null value
    }
}
===expect===
