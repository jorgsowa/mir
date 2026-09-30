===description===
inside method body
===file===
<?php
class A {
    public function go(): void {
        missing();
//      ^^^^^^^^^ UndefinedFunction: Function missing() is not defined
    }
}
===expect===
