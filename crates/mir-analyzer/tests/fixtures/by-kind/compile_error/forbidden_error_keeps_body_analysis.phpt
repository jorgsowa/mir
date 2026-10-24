===description===
A compile-time error elsewhere in the file does not skip body analysis
===file===
<?php
class A {
    public function __callStatic($name, $args) {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Compile error: Method A::__callStatic() must be static [magic-method-static]
    public function f(): void {
        $u = $undefinedVar;
//      ^^^^^^^^^^^^^^^^^^ MixedAssignment: Variable $u is assigned a mixed type
//           ^^^^^^^^^^^^^ UndefinedVariable: Variable $undefinedVar is not defined
        echo $u;
    }
}
