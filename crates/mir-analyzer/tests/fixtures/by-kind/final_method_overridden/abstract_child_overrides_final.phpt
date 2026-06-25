===description===
FinalMethodOverridden fires when an abstract child class overrides a final method from its parent.
===file===
<?php
class ParentClass {
    final public function locked(): void {}
}
abstract class Child extends ParentClass {
    public function locked(): void {}
}
===expect===
FinalMethodOverridden@6:4-6:37: Method Child::locked() cannot override final method from ParentClass
