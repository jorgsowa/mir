===description===
Final built-in classes PHP declares `@strict-properties` throw; a non-final one (`DateException`) may hold a subclass, which only deprecates.
===config===
<mir>
  <phpVersion>8.3</phpVersion>
</mir>
===file===
<?php
final class AppDateException extends DateException {}

function writes(
    WeakMap $map,
    Closure $fn,
    Generator $gen,
    Random\Randomizer $random,
    DateException $date,
    AppDateException $app,
): void {
    $map->x = 1;
//  ^^^^^^^^^^^ InvalidDynamicProperty: Cannot create dynamic property WeakMap::$x
    $fn->x = 1;
//  ^^^^^^^^^^ InvalidDynamicProperty: Cannot create dynamic property Closure::$x
    $gen->x = 1;
//  ^^^^^^^^^^^ InvalidDynamicProperty: Cannot create dynamic property Generator::$x
    $random->x = 1;
//  ^^^^^^^^^^^^^^ InvalidDynamicProperty: Cannot create dynamic property Random\Randomizer::$x
    $date->x = 1;
//  ^^^^^^^^^^^^ DynamicPropertyCreation: Creation of dynamic property DateException::$x is deprecated
    $app->x = 1;
//  ^^^^^^^^^^^ DynamicPropertyCreation: Creation of dynamic property AppDateException::$x is deprecated
}
