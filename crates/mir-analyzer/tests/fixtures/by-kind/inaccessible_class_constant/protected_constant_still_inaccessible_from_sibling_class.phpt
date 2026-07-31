===description===
Negative control for the K3 fix: sibling classes (both extending a common
ancestor, neither extending the other) must still be denied access to each
other's protected constants.
===config===
suppress=UnusedParam
===file===
<?php
class Base {
}

class Child extends Base {
    protected const SECRET = 'hidden';
}

class Cousin extends Base {
    public function peek(): string {
        return Child::SECRET;
    }
}
===expect===
InaccessibleClassConstant@11:22-11:28: Cannot access constant Child::SECRET
