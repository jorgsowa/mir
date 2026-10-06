===description===
cross file trait method body
===file:Trait.php===
<?php
trait MyTrait {
    public function go(): void {
        missing_function();
//      ^^^^^^^^^^^^^^^^^^ UndefinedFunction: Function missing_function() is not defined
    }
}
===file:User.php===
<?php
class MyClass {
    use MyTrait;
}
