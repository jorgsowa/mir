===description===
does not report public method
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
class Foo {
//<^ +2:1 UnusedClass: Class Foo is never referenced
    public function publicMethod(): void {}
}
