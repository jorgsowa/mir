===description===
A class without @psalm-immutable can freely assign to $this properties — no error.
===config===
suppress=MissingPropertyType
===file===
<?php

class Counter {
    public int $value = 0;

    public function increment(): void {
        $this->value++;
    }

    public function reset(): void {
        $this->value = 0;
    }
}
===expect===
