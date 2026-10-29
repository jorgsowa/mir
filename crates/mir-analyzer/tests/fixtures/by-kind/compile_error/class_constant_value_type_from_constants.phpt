===description===
A typed class constant built from constants is checked against its type when PHP evaluates it while compiling the class.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
</mir>
===file===
<?php
namespace App;

const GREETING = 'hello';

final class Settings {
    const NAME = 'x';
    const NOTHING = null;

    const int FROM_OWN = self::NAME;
//                       ^^^^^^^^^^ CompileError: Cannot use string as value for class constant App\Settings::FROM_OWN of type int [class-constant-value-type]
    const ?int NULLABLE = self::NAME;
//                        ^^^^^^^^^^ CompileError: Cannot use string as value for class constant App\Settings::NULLABLE of type ?int [class-constant-value-type]
    const int NOT_NULLABLE = self::NOTHING;
//                           ^^^^^^^^^^^^^ CompileError: Cannot use null as value for class constant App\Settings::NOT_NULLABLE of type int [class-constant-value-type]
    const int BUILTIN = \PHP_EOL;
//                      ^^^^^^^^ CompileError: Cannot use string as value for class constant App\Settings::BUILTIN of type int [class-constant-value-type]
    const int|string OK = self::NAME . 'y';
    const int RUNTIME = GREETING;
}
