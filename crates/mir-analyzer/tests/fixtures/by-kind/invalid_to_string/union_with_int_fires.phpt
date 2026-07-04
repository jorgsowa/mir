===description===
__toString() with string|int union return type fires — not all atoms are string
===config===
php_version=8.0
===file===
<?php
class UnionReturn {
    /** @return string|int */
    public function __toString() {
        return 42;
    }
}
new UnionReturn();
===expect===
InvalidToString@4:33-6:5: Method UnionReturn::__toString() must return a string
