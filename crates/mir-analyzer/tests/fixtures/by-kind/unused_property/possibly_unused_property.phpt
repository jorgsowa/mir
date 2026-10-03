===description===
Possibly unused property
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
final class A {
    public string $foo = "hello";
}

new A();
===expect===
