===description===
Multi-level inheritance: grandparent has @property magic, grandchild declares a
real property with the same name. The intermediate ancestor has no own property.
No OverriddenPropertyAccess should be emitted.
===config===
<mir>
  <phpVersion>8.2</phpVersion>
</mir>
===file===
<?php

/** @property string $value */
class GrandParent {
    public function __get(string $key): mixed { return null; }
}

class Parent_ extends GrandParent {}

class Child extends Parent_ {
    private string $value = '';
}
