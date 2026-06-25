===description===
FinalMethodOverridden fires when a concrete child overrides a final method declared in an abstract parent.
===file===
<?php
abstract class ParentClass {
    final public function locked(): void {}
}
class Child extends ParentClass {
    public function locked(): void {}
}
===expect===
FinalMethodOverridden@6:4-6:37: Method Child::locked() cannot override final method from ParentClass
