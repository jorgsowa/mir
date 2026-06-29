===description===
__toString() with a bool return type fires InvalidToString (bool is not a string)
===file===
<?php
class BoolReturn {
    public function __toString(): bool {
        return true;
    }
}
new BoolReturn();
===expect===
InvalidToString@3:39-5:40: Method BoolReturn::__toString() must return a string
