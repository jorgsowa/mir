===description===
InvalidToString fires when __toString returns a non-string type.
===file===
<?php
class Counter {
    private int $count = 0;

    public function __toString(): int {
//                                    ^ +2:5 InvalidToString: Method Counter::__toString() must return a string
        return $this->count;
    }
}
===expect===
