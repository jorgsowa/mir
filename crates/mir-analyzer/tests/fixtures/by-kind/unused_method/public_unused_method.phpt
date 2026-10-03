===description===
Public unused method
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
final class A {
    /** @return void */
    public function foo() {}
}

new A();
===expect===
