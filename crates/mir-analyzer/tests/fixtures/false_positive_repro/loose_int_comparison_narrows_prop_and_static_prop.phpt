===description===
Loose `==`/`!=` int comparison (`$this->prop == 42`, `self::$prop != 42`)
narrows a property/static-property receiver, mirroring the already-existing
plain-variable narrowing — property/static-property receivers previously
stayed unnarrowed (tracked as W5).
===config===
suppress=UnusedVariable,MissingPropertyType,UnusedParam
===file===
<?php
class Box {
    /** @var int|null */
    public $count;

    /** @var int|null */
    public static $scount;

    public function takesInt(int $x): void {}
    public static function takesIntStatic(int $x): void {}

    public function checkPropEquals(): void {
        if ($this->count == 42) {
            $this->takesInt($this->count);
        }
    }

    public function checkPropNotEquals(): void {
        if ($this->count != 42) {
            $_ = 1;
        } else {
            $this->takesInt($this->count);
        }
    }

    public static function checkStaticPropEquals(): void {
        if (self::$scount == 42) {
            self::takesIntStatic(self::$scount);
        }
    }

    // `null == 0` is PHP's one surprising loose-equality case — must stay
    // conservative and NOT narrow away null here.
    public function checkPropEqualsZeroStaysUnnarrowed(): void {
        if ($this->count == 0) {
            $this->takesInt($this->count);
        }
    }
}
===expect===
PossiblyNullArgument@36:28-36:40: Argument $x of takesInt() might be null
