===description===
Constants declared in a trait keep their inferred, docblock and native types through self::, static::, the using class and subclasses.
===file===
<?php
trait Inner {
    public const NAME = 'inner';
}

trait Limits {
    use Inner;

    public const MAX = 5;
    /** @var 'a'|'b' */
    public const MODE = 'a';
    public const int TYPED = 7;

    public function inTrait(): void {
        $max = self::MAX;
        /** @mir-check $max is 5 */
        $late = static::MAX;
        /** @mir-check $late is 5 */
        $nested = self::NAME;
        /** @mir-check $nested is 'inner' */
        echo $max, $late, $nested;
    }
}

class Base {
    use Limits;

    public function inClass(): void {
        $max = self::MAX;
        /** @mir-check $max is 5 */
        $mode = self::MODE;
        /** @mir-check $mode is 'a'|'b' */
        $typed = self::TYPED;
        /** @mir-check $typed is 7 */
        echo $max, $mode, $typed;
    }
}

class Child extends Base {
    public function inChild(): void {
        $max = self::MAX;
        /** @mir-check $max is 5 */
        $viaParent = parent::MAX;
        /** @mir-check $viaParent is 5 */
        $nested = self::NAME;
        /** @mir-check $nested is 'inner' */
        echo $max, $viaParent, $nested;
    }
}

function outside(Child $c): void {
    $max = Base::MAX;
    /** @mir-check $max is 5 */
    $nested = Child::NAME;
    /** @mir-check $nested is 'inner' */
    $viaObject = $c::MAX;
    /** @mir-check $viaObject is 5 */
    echo $max, $nested, $viaObject;
}
