===description===
does not report called private method
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
class Foo {
//<^ +6:1 UnusedClass: Class Foo is never referenced
    public function run(): void {
        $this->helper();
    }

    private function helper(): void {}
}
