===description===
`isset($this->prop)` narrows the property to non-null, the property-receiver
counterpart of `isset($x)` narrowing a plain variable — `isset()` is false
for both an unset and a null-valued property, so a true result proves both.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Foo {
    public function realMethod(): void {}
}

final class Holder {
    public ?Foo $foo = null;

    public function narrows(): void {
        if (isset($this->foo)) {
            $this->foo->realMethod();
        }
    }

    public function stillFlaggedWithoutGuard(): void {
        $this->foo->realMethod();
//      ^^^^^^^^^^^^^^^^^^^^^^^^ PossiblyNullMethodCall: Cannot call method realMethod() on possibly null value
    }
}
===expect===
