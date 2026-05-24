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
UndefinedFunction@4:9: Function missing_function() is not defined
