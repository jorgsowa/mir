===description===
InvalidStaticInvocation fires when the non-static method is inherited from a parent class.
===file===
<?php
class Base {
    public function render(): void {}
}

class View extends Base {}

View::render();
===expect===
InvalidStaticInvocation@8:0-8:14: Non-static method View::render() cannot be called statically
