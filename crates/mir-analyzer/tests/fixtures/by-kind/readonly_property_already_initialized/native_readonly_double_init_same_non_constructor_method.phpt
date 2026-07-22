===description===
Native `readonly` allows initialization from any method of the declaring
class, not just the constructor — but two writes to the same property
within that SAME method are still only the first-one-legal.
===config===
suppress=MissingConstructor
===file===
<?php
class Counter {
    public readonly int $value;

    public function init(int $v): void {
        $this->value = $v;
        $this->value = $v + 1;
    }
}
===expect===
ReadonlyPropertyAlreadyInitialized@7:8-7:29: Cannot modify readonly property Counter::$value — already initialized
