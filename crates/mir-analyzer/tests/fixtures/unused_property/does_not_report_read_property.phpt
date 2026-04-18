===source===
<?php
class Foo {
    private string $name = 'bar';

    public function getName(): string {
        return $this->name;
    }
}
===expect===
