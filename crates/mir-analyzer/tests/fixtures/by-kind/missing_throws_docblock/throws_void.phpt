===description===
@throws void reports every escaping exception, unchecked ones included
===file===
<?php
final class Boom extends \RuntimeException {}
final class Checked extends \Exception {}

/** @phpstan-throws void */
function claimsQuiet(): void {
    throw new \RuntimeException('nope');
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ MissingThrowsDocblock: Exception RuntimeException is thrown but not declared in @throws
}

function undocumentedThrow(): void {
    throw new \RuntimeException('undocumented');
}

/** @throws Boom */
function documented(): void {
    throw new Boom('x');
}

/** @throws void */
function callsDocumented(): void {
    documented();
//  ^^^^^^^^^^^^ MissingThrowsDocblock: Exception Boom is thrown but not declared in @throws
}

/** @throws void */
function catchesEverything(): void {
    try {
        documented();
        throw new Boom('y');
    } catch (Boom $e) {
        echo $e->getMessage();
    }
}

/** @throws void */
function closureBodyIsSeparate(): \Closure {
    return function (): void {
        throw new Boom('later');
    };
}

/** @throws Checked */
function declaresChecked(): void {
    throw new Checked('c');
}

function missesChecked(): void {
    declaresChecked();
//  ^^^^^^^^^^^^^^^^^ MissingThrowsDocblock: Exception Checked is thrown but not declared in @throws
}

final class Service {
    /** @throws Boom */
    public static function risky(): void {
        throw new Boom('s');
    }

    /** @throws void */
    public function quiet(): void {
        self::risky();
//      ^^^^^^^^^^^^^ MissingThrowsDocblock: Exception Boom is thrown but not declared in @throws
    }

    public function loud(): void {
        self::risky();
    }
}
