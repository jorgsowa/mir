===description===
A private __clone() names its declaring class and the calling scope, as PHP's error does.
===file===
<?php
class Base {
    private function __clone() {}
}

class Child extends Base {
    public function copyBase(Base $base): Base {
        return clone $base;
//             ^^^^^^^^^^^ InvalidClone: Call to private method Base::__clone() from scope Child
    }
}

function copyChild(Child $child): Child {
    return clone $child;
//         ^^^^^^^^^^^^ InvalidClone: Call to private method Base::__clone() from global scope
}
