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
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ FinalMethodOverridden: Method Child::alpha() cannot override final method from ParentClass
    public function beta(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ FinalMethodOverridden: Method Child::beta() cannot override final method from ParentClass
}
