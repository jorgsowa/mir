===description===
does not count read after throw expression
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
UnusedProperty@1:0: Private property Foo::$name is never read
UnreachableCode@7:8: Unreachable code detected
===ignore===
TODO
