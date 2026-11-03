===description===
does not count read after throw expression
===file===
<?php
class Foo {
//<^ +7:1 UnusedClass: Class Foo is never referenced
    private string $name = 'bar';
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedProperty: Private property Foo::$name is never read

    public function getName(): string {
        $value = throw new RuntimeException('stop');
        return $this->name;
//      ^^^^^^^^^^^^^^^^^^^ UnreachableCode: Unreachable code detected
    }
}
