===description===
match/switch subject narrowing only handled a bare variable subject; a
property or static-property subject (single-condition arms, and
fallthrough `case 'a': case 'b':`) now narrows the same way. Last function
proves genuine mismatches on a property subject are still caught, not
blanket-suppressed.
===config===
suppress=UnusedParam,MissingConstructor,MissingReturnType
===file===
<?php
class Holder {
    public int|string $id = 0;
    public static int|string $sid = 0;

    public function viaMatch(): string {
        return match ($this->id) {
            'a', 'b' => strtoupper($this->id),
            default => 'x',
        };
    }

    public static function viaMatchStatic(): string {
        return match (self::$sid) {
            'a', 'b' => strtoupper(self::$sid),
            default => 'x',
        };
    }

    public function viaSwitchSingleCase(): string {
        switch ($this->id) {
            case 'a':
                return strtoupper($this->id);
            default:
                return 'x';
        }
    }

    public function viaSwitchFallthrough(): string {
        switch ($this->id) {
            case 'a':
            case 'b':
                return strtoupper($this->id);
            default:
                return 'x';
        }
    }

    public static function viaSwitchFallthroughStatic(): string {
        switch (self::$sid) {
            case 'a':
            case 'b':
                return strtoupper(self::$sid);
            default:
                return 'x';
        }
    }

    public function stillCatchesRealMismatch(): string {
        return match ($this->id) {
            true => 'never',
            default => 'x',
        };
    }
}
===expect===
TypeDoesNotContainType@51:12-51:16: Type 'int|string' can never contain type 'true'
