===description===
An int-literal class constant bounds a comparison like a literal does
===file===
<?php
class Limits {
    public const MAX = 20;
    public const NAME = 'x';
}

class Page extends Limits {
    private const int LOW = 1;
    /** @var int<1, 20> */
    public int $limit = 1;
    public int $free = 0;
    public static int $shared = 0;

    public function selfConst(int $n): void {
        if ($n > self::MAX || $n < 1) {
            return;
        }
        /** @mir-check $n is int<1, 20> */
        $this->limit = $n;
    }

    public function parentConst(int $n): void {
        if ($n > parent::MAX || $n < self::LOW) {
            return;
        }
        /** @mir-check $n is int<1, 20> */
        $this->limit = $n;
    }

    public function namedClassConst(int $n): void {
        if (Limits::MAX < $n || 1 > $n) {
            return;
        }
        /** @mir-check $n is int<1, 20> */
        $this->limit = $n;
    }

    public function propertyOperand(): void {
        if ($this->free > self::MAX) {
            return;
        }
        /** @mir-check $this->free is int<min, 20> */
        $this->free;
    }

    public function staticPropOperand(): void {
        if (self::$shared > self::MAX) {
            return;
        }
        /** @mir-check self::$shared is int<min, 20> */
        self::$shared;
    }

    public function unrelatedBranchStillErrors(int $n): void {
        if ($n > self::MAX) {
            return;
        }
        $this->limit = $n;
//      ^^^^^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $limit expects 'int<1, 20>', cannot assign 'int<min, 20>'
    }

    public function lateStaticBindingIsNotNarrowed(int $n): void {
        if ($n > static::MAX || $n < 1) {
            return;
        }
        $this->limit = $n;
//      ^^^^^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $limit expects 'int<1, 20>', cannot assign 'int<1, max>'
    }

    public function nonIntConstantIgnored(int $n): void {
        if ($n > self::NAME) {
            return;
        }
        $this->limit = $n;
//      ^^^^^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $limit expects 'int<1, 20>', cannot assign 'int'
    }
}
