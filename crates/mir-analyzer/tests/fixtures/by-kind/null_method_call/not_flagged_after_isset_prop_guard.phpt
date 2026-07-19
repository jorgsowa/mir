===description===
`isset($this->prop)` narrows the property to non-null, the property-receiver
counterpart of `isset($x)` narrowing a plain variable — `isset()` is false
for both an unset and a null-valued property, so a true result proves both.
===config===
suppress=UnusedParam
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
    }
}
===expect===
PossiblyNullMethodCall@16:8-16:32: Cannot call method realMethod() on possibly null value
