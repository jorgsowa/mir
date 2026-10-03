===description===
does not report public method
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
class Foo {
    public function publicMethod(): void {}
}
===expect===
