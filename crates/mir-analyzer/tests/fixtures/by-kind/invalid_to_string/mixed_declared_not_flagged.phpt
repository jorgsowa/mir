===description===
FP guard: __toString() with mixed return type is NOT flagged (conservative — mixed is unknown)
===file===
<?php
class MixedReturn {
    /** @return mixed */
    public function __toString() {
        return "hello";
    }
}
new MixedReturn();
===expect===
