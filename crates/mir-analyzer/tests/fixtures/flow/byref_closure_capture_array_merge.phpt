===description===
A closure may run zero or many times, so a by-ref array capture merged back into the parent stays readable for keys the closure adds, whatever callee runs it and however the write is guarded.
===config===
<mir>
  <issueHandlers>
    <MissingClosureReturnType errorLevel="suppress"/>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
    <MissingReturnType errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function run(callable $f): void { $f(); }

class Runner {
    public function each(callable $f): void { $f(1); }
    public static function eachStatic(callable $f): void { $f(1); }
}

class Subject {
    public function viaFunction(): int {
        $sizes = [];
        run(function () use (&$sizes) { $sizes[] = 5; });
        return $sizes[0];
    }

    public function viaMethod(Runner $r): int {
        $sizes = [];
        $r->each(function (int $n) use (&$sizes) { $sizes[] = $n; });
        return $sizes[0];
    }

    public function viaStaticMethod(): int {
        $sizes = [];
        Runner::eachStatic(function (int $n) use (&$sizes) { $sizes[] = $n; });
        return $sizes[0];
    }

    public function guardedWrite(): int {
        $sizes = [];
        run(function () use (&$sizes) {
            if (rand() > 0) {
                $sizes[] = 5;
            }
        });
        return $sizes[0];
    }

    public function loopWrite(): int {
        $byKey = [];
        run(function () use (&$byKey) {
            foreach ([1, 2] as $i) {
                $byKey[$i] = $i;
            }
        });
        return $byKey[1];
    }

    public function addedKey(): int {
        $shape = ['count' => 0];
        $fill = function () use (&$shape) { $shape['extra'] = 1; };
        $fill();
        return $shape['extra'];
    }

    public function nestedClosure(): int {
        $sizes = [];
        $outer = function () use (&$sizes) {
            return function () use (&$sizes) { $sizes[] = 1; };
        };
        $outer()();
        return $sizes[0];
    }

    public function byValueStaysEmpty(): int {
        $sizes = [];
        run(function () use ($sizes) { $sizes[] = 5; });
        return $sizes[0];
//                    ^ NonExistentArrayOffset: Array offset '0' does not exist
    }

    public function noWriteStaysEmpty(): int {
        $sizes = [];
        run(function () use (&$sizes) { return count($sizes); });
        return $sizes[0];
//                    ^ NonExistentArrayOffset: Array offset '0' does not exist
    }
}
