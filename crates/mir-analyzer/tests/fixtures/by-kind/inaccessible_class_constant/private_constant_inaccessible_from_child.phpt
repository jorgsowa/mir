===description===
InaccessibleClassConstant fires when a child class tries to access a private constant from its parent.
===file===
<?php
class Base {
    private const SECRET = "top-secret";
}

class Child extends Base {
    public function getSecret(): string {
        return Base::SECRET;
    }
}
===expect===
InaccessibleClassConstant@8:21-8:27: Cannot access constant Base::SECRET
