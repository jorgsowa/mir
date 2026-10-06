===description===
Count narrowing on properties; optional keys widen the length range; a branch no shape can satisfy keeps the type.
===file===
<?php
class Box {
    /** @var array{0: int}|array{0: int, 1: int} */
    public array $pair = [0];
    /** @var array{0: int}|array{0: int, 1: int} */
    public static array $shared = [0];

    public function second(): int {
        if (count($this->pair) === 2) {
            return $this->pair[1];
        }
        return 0;
    }

    public function sharedSecond(): int {
        if (count(self::$shared) === 2) {
            return self::$shared[1];
        }
        return 0;
    }
}

/** @param array{0: int, 1?: int}|array{0: int, 1: int, 2: int} $a */
function optional(array $a): int {
    if (count($a) === 1) {
        /** @mir-check $a is array{0: int, 1?: int} */
        return $a[0];
    }
    return 0;
}

/** @param array{0: int}|array{0: int, 1: int} $a */
function deadBranchKeepsType(array $a): int {
    if (count($a) === 5) {
        return $a[0];
    }
    return $a[0];
}
