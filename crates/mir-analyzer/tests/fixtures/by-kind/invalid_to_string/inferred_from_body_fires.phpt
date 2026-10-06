===description===
__toString() with no declared return type fires when the body-inferred type is not string
===file===
<?php
class InferredReturn {
    public function __toString() {
//                               ^ +2:5 InvalidToString: Method InferredReturn::__toString() must return a string
        return 42;
    }
}
new InferredReturn();
