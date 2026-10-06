===description===
__toString() with a bool return type fires InvalidToString (bool is not a string)
===file===
<?php
class BoolReturn {
    public function __toString(): bool {
//                                     ^ +2:5 InvalidToString: Method BoolReturn::__toString() must return a string
        return true;
    }
}
new BoolReturn();
