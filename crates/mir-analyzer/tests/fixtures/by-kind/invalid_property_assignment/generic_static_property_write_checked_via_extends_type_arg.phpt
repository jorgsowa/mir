===description===
Writing to an inherited `@template T`-typed static property through a
subclass that binds `T` via `@extends Box<int>` must be checked against
the bound concrete type.
===config===
<mir>
  <issueHandlers>
    <MissingPropertyType errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php

/**
 * @template T
 */
class Box {
    /** @var T */
    public static $value;
}

/**
 * @extends Box<int>
 */
class IntBox extends Box {}

function bad(): void {
    IntBox::$value = 'not an int';
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $value expects 'int', cannot assign '"not an int"'
}
===expect===
