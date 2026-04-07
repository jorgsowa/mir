===source===
<?php
class A {
    public function go(): void {
        missing();
    }
}
===expect===
UndefinedFunction at 4:8
