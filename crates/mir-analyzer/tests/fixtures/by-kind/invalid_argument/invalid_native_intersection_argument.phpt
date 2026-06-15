===description===
Invalid native intersection argument
===file===
<?php
interface A {
    function foo(): void;
}
interface B {
}
class C implements A {
    function foo(): void {
    }
}
function test(A&B $in): void {
    $in->foo();
}
test(new C());
                
===expect===
InvalidArgument@14:5-14:12: Argument $in of test() expects 'A&B', got 'C'
