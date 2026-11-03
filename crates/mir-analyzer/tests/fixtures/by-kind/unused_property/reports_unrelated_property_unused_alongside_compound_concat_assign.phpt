===description===
A compound concat assign to one property does not exempt an unrelated unused
property on the same class.
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
class Foo {
//<^ +7:1 UnusedClass: Class Foo is never referenced
    private string $log = '';
    private string $unused = '';
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnusedProperty: Private property Foo::$unused is never read

    public function run(): void {
        $this->log .= 'x';
    }
}
