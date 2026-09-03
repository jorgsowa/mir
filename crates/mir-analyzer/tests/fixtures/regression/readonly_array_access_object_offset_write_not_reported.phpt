===description===
FP-K7: `$this->readonlyProp[$k] = $v` / `unset($this->readonlyProp[$k])` on
an object-typed readonly property that implements `ArrayAccess` dispatch to
`offsetSet`/`offsetUnset` on the already-held object — a method call, not a
reassignment of the property binding — so PHP allows both even though the
property is readonly (verified live). check_property_readonly_write treated
every array-index write through a property base as mutating the property
itself, regardless of whether the property actually held a plain array.
===config===
suppress=MissingConstructor,UnusedParam
===file===
<?php
class Store implements ArrayAccess {
    private array $data = [];
    public function offsetExists(string $offset): bool { return isset($this->data[$offset]); }
    public function offsetGet(string $offset): mixed { return $this->data[$offset] ?? null; }
    public function offsetSet(string $offset, mixed $value): void { $this->data[$offset] = $value; }
    public function offsetUnset(string $offset): void { unset($this->data[$offset]); }
}

class Holder {
    public function __construct(
        public readonly Store $store,
    ) {}

    public function set(string $k, mixed $v): void {
        $this->store[$k] = $v;
    }

    public function remove(string $k): void {
        unset($this->store[$k]);
    }
}
===expect===
