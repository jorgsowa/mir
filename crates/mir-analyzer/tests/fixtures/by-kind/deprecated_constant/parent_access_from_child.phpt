===description===
DeprecatedConstant fires when parent:: accesses a deprecated constant from a child class.
===file===
<?php
class Base {
    /** @deprecated use BASE_NEW instead */
    const OLD = 1;
}

class Child extends Base {
    public function legacy(): void {
        echo parent::OLD;
    }
}
===expect===
DeprecatedConstant@9:21-9:24: Constant Base::OLD is deprecated: use BASE_NEW instead
