===description===
Negative controls for the L4 never-inference fix: a body with no return
statement still infers `void` when it can genuinely fall off the end
(no throw at all, or a throw on only SOME paths) — only a body that NEVER
falls through infers `never`. Both calls here must still be flagged: void
must not get silently widened to never (the bottom type, which would
satisfy any parameter and suppress the check entirely).
===config===
suppress=UnusedParam,MissingParamType
===file===
<?php

class Service {
    public function plain($n) {
        // No throw at all — ordinary void.
    }

    public function conditional($n) {
        // Only ONE path throws — falling through the other is still
        // possible, so this must stay void, not never.
        if ($n) {
            throw new \LogicException('bad');
        }
    }
}

function run(string $sql): void {}

function usePlain(Service $s, $n): void {
    run($s->plain($n));
}

function useConditional(Service $s, $n): void {
    run($s->conditional($n));
}
===expect===
InvalidArgument@20:8-20:21: Argument $sql of run() expects 'string', got 'void'
InvalidArgument@24:8-24:27: Argument $sql of run() expects 'string', got 'void'
