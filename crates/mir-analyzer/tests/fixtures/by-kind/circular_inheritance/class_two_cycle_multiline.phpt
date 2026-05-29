===description===
class two cycle with multi-line body (line_end/col_end must clamp to declaration line)
===file===
<?php
class A extends B
{
    public function foo(): void {}
}
class B extends A
{
    public function bar(): void {}
}
===expect===
CircularInheritance@6:0-6:17: Class B has a circular inheritance chain
