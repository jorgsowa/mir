===file===
<?php
class A {
    public function go(): void {
        missing();
    }
}
===expect===
UndefinedFunction: Function missing() is not defined
