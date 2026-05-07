===description===
inside method body
===file===
<?php
class A {
    public function go(): void {
        missing();
    }
}
===expect===
UndefinedFunction@4:8: Function missing() is not defined
