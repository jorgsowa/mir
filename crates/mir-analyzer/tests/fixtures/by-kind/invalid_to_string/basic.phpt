===description===
InvalidToString fires when __toString returns a non-string type.
===file===
<?php
class Counter {
    private int $count = 0;

    public function __toString(): int {
        return $this->count;
    }
}
===expect===
InvalidToString@5:38-7:39: Method Counter::__toString() must return a string
