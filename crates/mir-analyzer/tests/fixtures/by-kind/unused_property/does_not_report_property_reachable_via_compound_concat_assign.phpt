===description===
A private property whose only reference is a compound concat assign
($this->log .= 'x') must not be reported unused.
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
class Foo {
//<^ +6:1 UnusedClass: Class Foo is never referenced
    private string $log = '';

    public function run(): void {
        $this->log .= 'x';
    }
}
