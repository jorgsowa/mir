===description===
Possibly unused property written never read
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
final class A {
    public string $foo = "hello";
}

$a = new A();
$a->foo = "bar";
