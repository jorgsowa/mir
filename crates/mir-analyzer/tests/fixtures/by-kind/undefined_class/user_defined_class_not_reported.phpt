===description===
user defined class not reported
===file===
<?php
class MyClass {}
function test(): void {
    new MyClass();
}
===expect===
