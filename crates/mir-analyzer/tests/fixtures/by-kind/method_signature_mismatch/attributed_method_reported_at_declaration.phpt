===description===
An attributed method is reported at its declaration, not at the attribute line.
===file===
<?php
class Base { public function run(int $a): void { echo $a; } }
final class Child extends Base {
    #[\Override]
    public function run(string $a): void {}
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MethodSignatureMismatch: Method Child::run() signature mismatch: parameter $a type 'string' is incompatible with parent type 'int'
}
