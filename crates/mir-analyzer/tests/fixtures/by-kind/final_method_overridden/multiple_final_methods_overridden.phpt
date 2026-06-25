===description===
FinalMethodOverridden fires separately for each overridden final method.
===file===
<?php
class ParentClass {
    final public function alpha(): void {}
    final public function beta(): void {}
}
class Child extends ParentClass {
    public function alpha(): void {}
    public function beta(): void {}
}
===expect===
FinalMethodOverridden@7:4-7:36: Method Child::alpha() cannot override final method from ParentClass
FinalMethodOverridden@8:4-8:35: Method Child::beta() cannot override final method from ParentClass
