===description===
Assert if true method call
===config===
suppress=MixedArgument
===file===
<?php
class C {
    /**
     * @param mixed $p
     * @assert-if-true int $p
     */
    public function isInt($p): bool {
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
