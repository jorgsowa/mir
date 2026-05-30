===description===
does not report assignment in constructor
===file===
<?php
class Foo {
    public readonly string $name;

    public function __construct(string $name) {
        $this->name = $name;
    }
}
===expect===
