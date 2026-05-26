===description===
bare generic property accepts single parameterized type
===file===
<?php
/** @template T */
class Box {}

class Container {
    private Box $item;

    public function set(Box $b): void {
        $this->item = $b;
    }

    public function setSingle(): void {
        /** @var Box<string> $value */
        $value = new Box();
        $this->item = $value;
    }
}
===expect===
