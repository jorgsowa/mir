===description===
A compile-time error elsewhere in the file does not skip body analysis
===file===
<?php
class A {
    public function __callStatic($name, $args) {}
    public function f(): void {
        $u = $undefinedVar;
        echo $u;
    }
}
===expect===
CompileError@3:4-3:49: Compile error: Method A::__callStatic() must be static [magic-method-static]
MixedAssignment@5:8-5:26: Variable $u is assigned a mixed type
UndefinedVariable@5:13-5:26: Variable $undefinedVar is not defined
