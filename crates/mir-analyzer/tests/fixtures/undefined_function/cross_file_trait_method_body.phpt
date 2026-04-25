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
Trait.php: UndefinedFunction: Function missing_function() is not defined
