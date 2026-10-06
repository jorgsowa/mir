===description===
FP-J(b): A child class declaring a real property with the same name as a parent
@property magic docblock tag must NOT emit OverriddenPropertyAccess. Magic
@property declarations carry no PHP native type and are not real inherited
properties, so PHP enforces no visibility rules against them.
===config===
<mir>
  <phpVersion>8.2</phpVersion>
</mir>
===file===
<?php

/**
 * @property string $name
 * @property-read int $id
 */
class Base {
    public function __get(string $key): mixed { return null; }
}

class Child extends Base {
    // Real property — @property in parent is not a real PHP property,
    // so no visibility rule is violated.
    private string $name = '';
    protected int $id = 0;
}
