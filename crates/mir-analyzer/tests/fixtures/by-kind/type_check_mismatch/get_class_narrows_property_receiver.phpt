===description===
get_class($this->prop) === 'ClassName' / === Foo::class narrows the property,
the property-receiver counterpart of get_class narrowing a plain variable —
gettype()/get_debug_type()/get_parent_class() already had this via
ScalarArgTarget, get_class() was the one left var-only.
===config===
suppress=UnusedVariable,UnusedParam,MissingPropertyType
===file===
<?php
class Foo {}
class Bar {}

final class Holder {
    /** @var Foo|Bar */
    public $obj;

    public function narrowsStringLiteral(): void {
        if (get_class($this->obj) === 'Foo') {
            /** @mir-check $this->obj is Foo */
            $_ = 1;
        }
    }

    public function narrowsStringLiteralSymmetric(): void {
        if ('Foo' === get_class($this->obj)) {
            /** @mir-check $this->obj is Foo */
            $_ = 1;
        }
    }

    public function narrowsClassConst(): void {
        if (get_class($this->obj) === Foo::class) {
            /** @mir-check $this->obj is Foo */
            $_ = 1;
        }
    }

    public function narrowsClassConstSymmetric(): void {
        if (Foo::class === get_class($this->obj)) {
            /** @mir-check $this->obj is Foo */
            $_ = 1;
        }
    }

    public function narrowsLooseStringLiteral(): void {
        if (get_class($this->obj) == 'Foo') {
            /** @mir-check $this->obj is Foo */
            $_ = 1;
        }
    }

    public function narrowsLooseStringLiteralSymmetric(): void {
        if ('Foo' == get_class($this->obj)) {
            /** @mir-check $this->obj is Foo */
            $_ = 1;
        }
    }
}
===expect===
