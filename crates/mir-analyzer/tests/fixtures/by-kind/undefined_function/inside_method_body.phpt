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
UndefinedFunction@4:9-4:18: Function missing() is not defined
