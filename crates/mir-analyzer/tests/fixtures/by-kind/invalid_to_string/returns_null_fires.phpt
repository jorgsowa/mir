===description===
__toString() with a null return type fires InvalidToString (null is not a string)
===config===
php_version=8.0
===file===
<?php
class NullReturn {
    public function __toString(): null {
        return null;
    }
}
new NullReturn();
===expect===
InvalidToString@3:39-5:5: Method NullReturn::__toString() must return a string
