===description===
does not report read property
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
class Foo {
//<^ +6:1 UnusedClass: Class Foo is never referenced
    private string $name = 'bar';

    public function getName(): string {
        return $this->name;
    }
}
