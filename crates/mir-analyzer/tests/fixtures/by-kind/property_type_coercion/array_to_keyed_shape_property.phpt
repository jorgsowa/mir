===description===
A generic array assigned to a keyed-shape property is a coercion; a value that cannot hold the shape is still invalid.
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Holder {
    /** @var array{id: int, name: string} */
    private array $meta = ['id' => 1, 'name' => 'x'];

    /** @var array{id: int}|null */
    private ?array $maybe = null;

    public function bare(array $row): void {
        $this->meta = $row;
//      ^^^^^^^^^^^^^^^^^^ PropertyTypeCoercion: Property $meta expects 'array{'id': int, 'name': string}', cannot assign 'array' — coercion may fail at runtime
    }

    /** @param array<string, int|string> $row */
    public function wide(array $row): void {
        $this->meta = $row;
//      ^^^^^^^^^^^^^^^^^^ PropertyTypeCoercion: Property $meta expects 'array{'id': int, 'name': string}', cannot assign 'array<string, int|string>' — coercion may fail at runtime
    }

    public function nullable(array $row): void {
        $this->maybe = $row;
//      ^^^^^^^^^^^^^^^^^^^ PropertyTypeCoercion: Property $maybe expects 'array{'id': int}|null', cannot assign 'array' — coercion may fail at runtime
    }

    /** @param array<string, int> $row */
    public function narrowValues(array $row): void {
        $this->meta = $row;
//      ^^^^^^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $meta expects 'array{'id': int, 'name': string}', cannot assign 'array<string, int>'
    }

    /** @param array<int, string> $row */
    public function wrongKeys(array $row): void {
        $this->meta = $row;
//      ^^^^^^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $meta expects 'array{'id': int, 'name': string}', cannot assign 'array<int, string>'
    }

    public function scalar(string $row): void {
        $this->meta = $row;
//      ^^^^^^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $meta expects 'array{'id': int, 'name': string}', cannot assign 'string'
    }
}
