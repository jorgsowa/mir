===description===
With catch clauses, code after try/catch/finally is reached only through a
completed try or a completed catch; an uncaught exception propagates past it.
finally's pessimistic mid-try seed must not leak onto that state.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
    <MissingThrowsDocblock errorLevel="suppress"/>
    <MixedReturnStatement errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
function risky(): int { return random_int(0, 1); }

function catchThrows(): int {
    try {
        $r = risky();
    } catch (\Throwable $e) {
        throw new \RuntimeException('failed', 0, $e);
    } finally {
        echo 'cleanup';
    }
    return $r;
}

function catchesReturnOrThrow(): int {
    try {
        $r = risky();
    } catch (\LogicException $e) {
        return 0;
    } catch (\Throwable $e) {
        throw $e;
    } finally {
        echo 'cleanup';
    }
    return $r;
}

function catchAssigns(): int {
    try {
        $r = risky();
    } catch (\Throwable $e) {
        $r = -1;
    } finally {
        echo 'cleanup';
    }
    return $r;
}

function finallyAssigns(): int {
    try {
        risky();
    } catch (\Throwable $e) {
        return 0;
    } finally {
        $done = 1;
    }
    return $done;
}

function finallyUnsets(): int {
    try {
        $r = risky();
    } catch (\Throwable $e) {
        return 0;
    } finally {
        unset($r);
    }
    return $r;
}

function catchFallsThrough(): int {
    try {
        $r = risky();
    } catch (\Throwable $e) {
        echo 'ignored';
    } finally {
        echo 'cleanup';
    }
    return $r;
}
===expect===
UndefinedVariable@58:11-58:13: Variable $r is not defined
PossiblyUndefinedVariable@69:11-69:13: Variable $r might not be defined
