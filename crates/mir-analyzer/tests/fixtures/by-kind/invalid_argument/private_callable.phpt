===description===
Private callable
===file===
<?php
class Base  {
    private function privateMethod() : void {}
}

class Example extends Base {
    public function test() : Closure {
        return Closure::fromCallable([$this, "privateMethod"]);
    }
}
===expect===
InvalidArgument
===ignore===
TODO
