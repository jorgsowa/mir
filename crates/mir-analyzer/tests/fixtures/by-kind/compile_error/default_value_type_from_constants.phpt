===description===
A typed property default built from constants is checked against the type: PHP rejects it while compiling the class when the value folds (the class's own earlier constants, `\BUILTIN` constants, internal class constants) and on first use otherwise (`InvalidPropertyAssignment`).
===config===
<mir>
  <phpVersion>8.4</phpVersion>
  <issueHandlers>
    <MissingConstructor errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
namespace App;

const GREETING = 'hello';

enum Suit { case Hearts; }

final class Defaults {
    const NAME = 'x';
    const COUNT = 1;
    const NOTHING = null;
    const ALIAS = self::NAME;

    public int $fromOwn = self::NAME;
//                        ^^^^^^^^^^ CompileError: Cannot use string as default value for property App\Defaults::$fromOwn of type int [default-value-type]
    public string $fromInt = self::COUNT;
//                           ^^^^^^^^^^^ CompileError: Cannot use int as default value for property App\Defaults::$fromInt of type string [default-value-type]
    public float $intWidens = self::COUNT;
    public ?int $nullable = self::NOTHING;
    public int $notNullable = self::NOTHING;
//                            ^^^^^^^^^^^^^ CompileError: Default value for property of type int may not be null. Use the nullable type ?int to allow null default value [default-value-type]
    public int|false $unionNull = self::NOTHING;
//                                ^^^^^^^^^^^^^ CompileError: Default value for property of type int|false may not be null. Use the nullable type int|false|null to allow null default value [default-value-type]
    public int $transitive = self::ALIAS;
//                           ^^^^^^^^^^^ CompileError: Cannot use string as default value for property App\Defaults::$transitive of type int [default-value-type]
    public int $qualified = Defaults::NAME;
//                          ^^^^^^^^^^^^^^ CompileError: Cannot use string as default value for property App\Defaults::$qualified of type int [default-value-type]
    public int $concat = self::NAME . 'y';
//                       ^^^^^^^^^^^^^^^^ CompileError: Cannot use string as default value for property App\Defaults::$concat of type int [default-value-type]
    public int $ternary = self::COUNT ? 'a' : 'b';
//                        ^^^^^^^^^^^^^^^^^^^^^^^ CompileError: Cannot use string as default value for property App\Defaults::$ternary of type int [default-value-type]
    public string $ok = self::NAME . 'y';
    public int $builtin = \PHP_EOL;
//                        ^^^^^^^^ CompileError: Cannot use string as default value for property App\Defaults::$builtin of type int [default-value-type]
    public string $internalClass = \PDO::ATTR_ERRMODE;
//                                 ^^^^^^^^^^^^^^^^^^ CompileError: Cannot use int as default value for property App\Defaults::$internalClass of type string [default-value-type]
    public static int $static = self::NAME;
//                              ^^^^^^^^^^ CompileError: Cannot use string as default value for property App\Defaults::$static of type int [default-value-type]
    public int $global = GREETING;
//                       ^^^^^^^^ InvalidPropertyAssignment: Property $global expects 'int', cannot assign 'string'
    public int $namespaceFallback = PHP_EOL;
//                                  ^^^^^^^ InvalidPropertyAssignment: Property $namespaceFallback expects 'int', cannot assign 'string'
    public string $enumCase = Suit::Hearts;
//                            ^^^^^^^^^^^^ InvalidPropertyAssignment: Property $enumCase expects 'string', cannot assign 'App\Suit'
    public int $declaredLater = self::LATER;
//                              ^^^^^^^^^^^ InvalidPropertyAssignment: Property $declaredLater expects 'int', cannot assign 'string'
    const LATER = 'l';
}
