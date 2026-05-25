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
MethodSignatureMismatch
===ignore===
TODO
