===description===
@psalm-self-out also retypes $this when the method is reached through
parent::/self::/static:: call syntax, not just `$obj->method()`.
===config===
suppress=UnusedParam
===file===
<?php
class Base {
    /** @psalm-self-out Ready */
    public function init(): void {}
}
class Ready extends Base {}

class Child extends Base {
    public function setup(): void {
        parent::init();
        /** @mir-check $this is Ready */
        $_ = 1;
    }
}
===expect===
