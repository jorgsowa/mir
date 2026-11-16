===description===
a closure passed to a @param-immediately-invoked-callable parameter throws at the call site
===file===
<?php
final class Boom extends \RuntimeException {}
final class Checked extends \Exception {}

final class Runner {
    /**
     * @param callable $callback
     * @param-immediately-invoked-callable $callback
     */
    public function runNow(callable $callback): void {
        $callback();
    }

    /** @param callable $callback */
    public function runLater(callable $callback): void {
        echo gettype($callback);
    }

    /**
     * @param-immediately-invoked-callable $callback
     */
    public static function runStatic(callable $callback): void {
        $callback();
    }
}

/** @phpstan-throws void */
function callsImmediate(Runner $runner): void {
    $runner->runNow(static function (): void { throw new Boom('now'); });
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MissingThrowsDocblock: Exception Boom is thrown but not declared in @throws
}

/** @phpstan-throws void */
function callsLater(Runner $runner): void {
    $runner->runLater(static function (): void { throw new Boom('later'); });
}

/** @phpstan-throws void */
function callsStatic(): void {
    Runner::runStatic(static fn () => throw new Boom('arrow'));
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MissingThrowsDocblock: Exception Boom is thrown but not declared in @throws
}

/** @phpstan-throws void */
function catchesInsideClosure(Runner $runner): void {
    $runner->runNow(static function (): void {
        try {
            throw new Boom('caught');
        } catch (Boom $e) {
            echo $e->getMessage();
        }
    });
}

/** @phpstan-throws void */
function catchesAroundCall(Runner $runner): void {
    try {
        $runner->runNow(static function (): void { throw new Boom('outer'); });
    } catch (Boom $e) {
        echo $e->getMessage();
    }
}

/**
 * @param callable $callback
 * @param-immediately-invoked-callable $callback
 */
function invokeNow(callable $callback): void {
    $callback();
}

/** @param callable $callback */
function plainFunction(callable $callback): void {
    $callback();
}

/** @throws Checked */
function declaresChecked(): void {
    invokeNow(static function (): void {
        throw new Checked('declared');
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MissingThrowsDocblock: Exception Checked is thrown but not declared in @throws
    });
}

function missesChecked(): void {
    invokeNow(callback: static function (): void {
//  ^ +2:6 MissingThrowsDocblock: Exception Checked is thrown but not declared in @throws
        throw new Checked('undeclared');
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MissingThrowsDocblock: Exception Checked is thrown but not declared in @throws
    });
}

function uncheckedStaysQuiet(): void {
    invokeNow(static function (): void { throw new Boom('unchecked'); });
    plainFunction(static function (): void { throw new Boom('plain'); });
}
