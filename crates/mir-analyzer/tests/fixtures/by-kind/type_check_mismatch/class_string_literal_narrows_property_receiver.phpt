===description===
`$this->prop === Foo::class` (a plain class-string comparison, not an enum
case) narrows the property, the property-receiver counterpart of the
existing plain-variable narrow_var_to_class_string. Also covers the
symmetric `Foo::class === $this->prop` form.
===config===
suppress=UnusedVariable,UnusedParam,MissingPropertyType
===file===
<?php
class Foo {}
class Bar {}

final class Holder {
    /** @var class-string<Foo>|class-string<Bar> */
    public $cls;

    public function narrows(): void {
        if ($this->cls === Foo::class) {
            /** @mir-check $this->cls is class-string<Foo> */
            $_ = 1;
        }
    }

    public function narrowsSymmetric(): void {
        if (Foo::class === $this->cls) {
            /** @mir-check $this->cls is class-string<Foo> */
            $_ = 1;
        }
    }
}
===expect===
