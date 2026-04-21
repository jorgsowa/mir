===source===
<?php
class Base {
    const VALUE = 10;
}
class Child extends Base {}
function test(): void {
    echo Child::VALUE;
}
===expect===
