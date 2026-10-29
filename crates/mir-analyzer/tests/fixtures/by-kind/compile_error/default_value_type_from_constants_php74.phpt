===description===
PHP 7.4 words a rejected property default built from constants by the declared type.
===config===
<mir>
  <phpVersion>7.4</phpVersion>
  <issueHandlers>
    <MissingConstructor errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
class Foo {}

final class Defaults {
    const NAME = 'x';
    const COUNT = 1;
    const NOTHING = null;

    public int $fromString = self::NAME;
//                           ^^^^^^^^^^ CompileError: Default value for property of type int can only be int [default-value-type]
    public ?string $nullable = self::COUNT;
//                             ^^^^^^^^^^^ CompileError: Default value for property of type string can only be string [default-value-type]
    public float $fromString2 = self::NAME;
//                              ^^^^^^^^^^ CompileError: Default value for property of type float can only be float or int [default-value-type]
    public iterable $iterable = self::COUNT;
//                              ^^^^^^^^^^^ CompileError: Default value for property of type iterable can only be an array [default-value-type]
    public Foo $object = self::COUNT;
//                       ^^^^^^^^^^^ CompileError: Property of type Foo may not have default value [default-value-type]
    public int $notNullable = self::NOTHING;
//                            ^^^^^^^^^^^^^ CompileError: Default value for property of type int may not be null. Use the nullable type ?int to allow null default value [default-value-type]
}
