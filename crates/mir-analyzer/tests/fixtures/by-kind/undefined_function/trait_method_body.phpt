===description===
trait method body
===file===
<?php
trait MyTrait {
    public function go(): void {
        missing_function();
    }
}
===expect===
UndefinedFunction@4:8-4:26: Function missing_function() is not defined
