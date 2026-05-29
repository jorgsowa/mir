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
UnusedProperty@3:4-3:32: Private property Foo::$name is never read
UnreachableCode@7:9-7:28: Unreachable code detected
