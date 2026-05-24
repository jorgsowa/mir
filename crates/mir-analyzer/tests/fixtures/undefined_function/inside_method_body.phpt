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
UndefinedFunction@4:9: Function missing() is not defined
