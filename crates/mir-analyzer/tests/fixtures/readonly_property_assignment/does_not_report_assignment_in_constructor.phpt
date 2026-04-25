===file===
<?php
class Foo {
    public readonly string $name;

    public function __construct(string $name) {
        $this->name = $name;
    }
}
===expect===
