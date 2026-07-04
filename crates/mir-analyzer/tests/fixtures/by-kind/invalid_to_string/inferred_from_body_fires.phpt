===description===
__toString() with no declared return type fires when the body-inferred type is not string
===file===
<?php
class InferredReturn {
    public function __toString() {
        return 42;
    }
}
new InferredReturn();
===expect===
InvalidToString@3:33-5:5: Method InferredReturn::__toString() must return a string
