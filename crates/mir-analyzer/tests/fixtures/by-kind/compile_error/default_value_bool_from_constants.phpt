===description===
From PHP 8.3 a boolean property default built from constants is named `true` or `false`; a class constant's is still `bool`.
===config===
<mir>
  <phpVersion>8.3</phpVersion>
  <issueHandlers>
    <MissingConstructor errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Flags {
    const ON = true;
    const OFF = false;

    public int $on = self::ON;
//                   ^^^^^^^^ CompileError: Cannot use true as default value for property Flags::$on of type int [default-value-type]
    public int $off = !self::ON;
//                    ^^^^^^^^^ CompileError: Cannot use false as default value for property Flags::$off of type int [default-value-type]
    const int C = self::OFF;
//                ^^^^^^^^^ CompileError: Cannot use bool as value for class constant Flags::C of type int [class-constant-value-type]
}
