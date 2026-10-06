===description===
trait method body
===file===
<?php
trait MyTrait {
    public function go(): void {
        missing_function();
//      ^^^^^^^^^^^^^^^^^^ UndefinedFunction: Function missing_function() is not defined
    }
}
