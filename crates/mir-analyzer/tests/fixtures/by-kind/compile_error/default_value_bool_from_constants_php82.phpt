===description===
Before PHP 8.3 a boolean property default built from constants is named `bool`.
===config===
<mir>
  <phpVersion>8.2</phpVersion>
  <issueHandlers>
    <MissingConstructor errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
final class Flags {
    const ON = true;

    public int $on = self::ON;
//                   ^^^^^^^^ CompileError: Cannot use bool as default value for property Flags::$on of type int [default-value-type]
}
