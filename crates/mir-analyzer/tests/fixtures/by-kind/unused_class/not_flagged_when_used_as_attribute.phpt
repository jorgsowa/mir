===description===
A final attribute class used only via #[MyAttr(...)] annotation elsewhere
must not be reported UnusedClass.
===config===
<mir>
  <findUnusedCode>true</findUnusedCode>
</mir>
===file===
<?php
#[Attribute]
final class MyAttr {
    public function __construct(public string $name = '') {}
}

#[MyAttr('x')]
final class Consumer {
}

new Consumer();
===expect===
