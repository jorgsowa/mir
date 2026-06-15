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
UndefinedFunction@4:8-4:17: Function missing() is not defined
