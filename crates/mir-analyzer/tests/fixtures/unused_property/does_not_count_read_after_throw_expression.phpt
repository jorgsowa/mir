===config===
find_dead_code=true
===file===
<?php
class Foo {
    private string $name = 'bar';

    public function getName(): string {
        $value = throw new RuntimeException('stop');
        return $this->name;
    }
}
===expect===
UnreachableCode: Unreachable code detected
UnusedProperty: Private property Foo::$name is never read
