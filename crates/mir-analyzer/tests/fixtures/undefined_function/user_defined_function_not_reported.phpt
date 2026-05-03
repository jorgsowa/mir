===description===
user defined function not reported
===file===
<?php
function myFn(): void {}
function test(): void {
    myFn();
}
===expect===
===ignore===
TODO
