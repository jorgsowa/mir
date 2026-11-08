===description===
A `@param` object type on a native `array` (or array type on a native class) is ignored in favour of the native type.
===file===
<?php
interface VL {}
class DD {
    public function get(): int { return 1; }
}
class F {
    /** @param VL $v */
    public function fmt(array $v): int { return count($v); }
    /** @param array $d */
    public function run(DD $d): int { return $this->fmt([1, 2]) + $d->get(); }
}
