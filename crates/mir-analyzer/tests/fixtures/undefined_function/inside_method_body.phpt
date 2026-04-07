===source===
<?php
class A {
    public function go(): void {
        missing();
    }
}
===expect===
UndefinedFunction: missing()
