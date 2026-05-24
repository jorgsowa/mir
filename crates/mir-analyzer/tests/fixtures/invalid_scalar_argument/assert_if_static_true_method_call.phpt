===description===
assertIfStaticTrueMethodCall
===file===
<?php
class C {
    /**
     * @param mixed $p
     * @psalm-assert-if-true int $p
     */
    public static function isInt($p): bool {
        return is_int($p);
    }
    /**
     * @param mixed $p
     */
    public function doWork($p): void {
        if ($this->isInt($p)) {
            strlen($p);
        }
    }
}
===expect===
InvalidScalarArgument
===ignore===
TODO
