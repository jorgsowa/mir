===config===
find_dead_code=true
===file===
<?php
class Foo {
    private string $name = 'bar';

    public function getName(): string {
        return $this->name;
    }
}
===expect===
