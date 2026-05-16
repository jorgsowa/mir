===description===
does not count read after throw expression
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
UnusedProperty@3:4: Private property Foo::$name is never read
UnreachableCode@7:8: Unreachable code detected
