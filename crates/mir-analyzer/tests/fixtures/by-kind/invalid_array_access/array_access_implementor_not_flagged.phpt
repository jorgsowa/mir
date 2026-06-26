===description===
InvalidArrayAccess does NOT fire when the object's class implements ArrayAccess
===config===
suppress=MixedArrayOffset
===file===
<?php
class Box implements \ArrayAccess
{
    private array $data = [];
    public function offsetExists(mixed $offset): bool { return isset($this->data[$offset]); }
    public function offsetGet(mixed $offset): mixed { return $this->data[$offset]; }
    public function offsetSet(mixed $offset, mixed $value): void { $this->data[$offset] = $value; }
    public function offsetUnset(mixed $offset): void { unset($this->data[$offset]); }
}
$box = new Box();
echo $box[0];
===expect===
