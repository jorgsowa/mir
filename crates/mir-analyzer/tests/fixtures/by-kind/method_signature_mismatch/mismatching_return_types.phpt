===description===
Mismatching return types
===file===
<?php
interface I1 {
  public function foo(): string;
}
interface I2 {
  public function foo(): int;
}
class A implements I1, I2 {
  public function foo(): string {
    return "hello";
  }
}
===expect===
MethodSignatureMismatch@9:2-9:33: Method A::foo() signature mismatch: return type 'string' is not a subtype of I2::foo() 'int'
