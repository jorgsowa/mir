===description===
A private method cannot be overridden, so `final` on it only draws a PHP warning.
===file===
<?php
class A {
    final private function f(): void {}
//                         ^ PrivateFinalMethod: Private method A::f() cannot be final as it is never overridden by other classes

    public function g(): void { $this->f(); }
}

trait T {
    private final function h(): void {}
//                         ^ PrivateFinalMethod: Private method T::h() cannot be final as it is never overridden by other classes
}
