===description===
FinalMethodOverridden fires for protected final methods; visibility does not exempt the override.
===file===
<?php
class ParentClass {
    final protected function locked(): void {}
}
class Child extends ParentClass {
    protected function locked(): void {}
}
===expect===
FinalMethodOverridden@6:4-6:40: Method Child::locked() cannot override final method from ParentClass
