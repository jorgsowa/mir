===description===
cross file trait method body
===file:Trait.php===
<?php
trait MyTrait {
    public function go(): void {
        missing_function();
    }
}
===file:User.php===
<?php
class MyClass {
    use MyTrait;
}
===expect===
Trait.php: UndefinedFunction@4:8-4:26: Function missing_function() is not defined
