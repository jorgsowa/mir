===description===
FinalMethodOverridden fires when a static final method is overridden.
===file===
<?php
class ParentClass {
    final public static function locked(): void {}
}
class Child extends ParentClass {
    public static function locked(): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ FinalMethodOverridden: Method Child::locked() cannot override final method from ParentClass
}
===expect===
